import { useEffect, useRef, useState } from "preact/hooks";
import { Checkbox, Input, NumberInput } from "./binded-input";
import {
  DatasetLayerIndex,
  LayerDescriptor,
  ParameterValue,
  SavedToolOutputAction,
  ToolDescriptor,
  ToolsScreenInfo,
} from "./bindings";
import { Dialog, useDialog } from "./dialog";
import { IndexedOptionPicker, OptionPicker } from "./option-picker";
import { client, state } from "./api";

export const ToolsScreen = ({ tools, layers }: ToolsScreenInfo) => {
  return (
    <div>
      <h2>Tools</h2>
      <div>
        {tools.map((tool, index) => (
          <ToolDialog tool={tool} index={index} layers={layers} />
        ))}
      </div>
    </div>
  );
};

type Param = { label: string } & (
  | { type: "Float"; value: number }
  | { type: "Int"; value: number }
  | { type: "String"; value: string }
  | { type: "Dataset"; value: number }
  | { type: "Layer"; value: DatasetLayerIndex; options: "Vector" | "Raster" }
  | { type: "Option"; value: string; options: string[] }
  | { type: "Flag"; value: boolean }
);

type ToolDialogProps = {
  tool: ToolDescriptor;
  index: number;
  layers: LayerDescriptor[];
};

const ToolDialog = ({ tool, index, layers }: ToolDialogProps) => {
  const { open, setOpen } = useDialog();

  const [params, setParams] = useState(() =>
    tool.inputs.map((input): Param => {
      switch (input.param_type.type) {
        case "Float":
          return {
            type: "Float",
            value: 0,
            label: input.label,
          };
        case "Int":
          return { type: "Int", value: 0, label: input.label };
        case "String":
          return { type: "String", value: "", label: input.label };
        case "Dataset":
          return { type: "Dataset", value: 0, label: input.label };
        case "Layer":
          return {
            type: "Layer",
            value: {
              dataset: 0,
              layer: { type: input.param_type.options, index: 0 },
            },
            label: input.label,
            options: input.param_type.options,
          };
        case "Option":
          return {
            type: "Option",
            value: input.param_type.options[0] ?? null,
            label: input.label,
            options: input.param_type.options,
          };
        case "Flag":
          return {
            type: "Flag",
            value: false,
            label: input.label,
          };
      }
    })
  );
  console.log(params);
  const setValueAt =
    <T,>(index: number) =>
    (value: T) => {
      const replacement = { ...params[index], value };
      const newArray = params.slice();
      newArray.splice(index, 1, replacement as any);
      setParams(newArray);
    };

  const makeParamBinding = <T extends Param>(param: T, index: number) => ({
    value: param.value,
    setValue: setValueAt(index),
  });

  const datasets = layers.reduce((arr, el) => {
    if (arr.includes(el.dataset_file)) {
      return arr;
    } else {
      return [...arr, el.dataset_file];
    }
  }, [] as string[]);

  const vectorLayers = layers.filter((layer) => layer.type === "Vector");
  console.log(vectorLayers);
  const rasterLayers = layers.filter((layer) => layer.type === "Raster");

  return (
    <Dialog openText={tool.label} modal={true} open={open} setOpen={setOpen}>
      <h3>{tool.label}</h3>
      {params.map((param) => {
        switch (param.type) {
          case "Float":
            return (
              <NumberInput
                label={param.label}
                binding={makeParamBinding(param, index)}
              />
            );
          case "Int":
            return (
              <NumberInput
                label={param.label}
                binding={makeParamBinding(param, index)}
              />
            );
          case "String":
            return (
              <Input
                label={tool.label}
                binding={makeParamBinding(param, index)}
              />
            );
          case "Dataset":
            return (
              <IndexedOptionPicker
                prompt={param.label}
                emptyText="No datasets to select"
                index={param.value}
                options={datasets}
                setIndex={setValueAt(index)}
              />
            );
          case "Layer":
            switch (param.options) {
              case "Vector":
                return (
                  <IndexedOptionPicker
                    prompt={param.label}
                    emptyText="There are no vector layers to select"
                    index={vectorLayers.findIndex(
                      (layer) =>
                        layer.dataset === param.value.dataset &&
                        layer.type === "Vector" &&
                        layer.index === param.value.layer.index
                    )}
                    options={vectorLayers.map((layer) => layer.dataset_file)}
                    setIndex={(layer_index) =>
                      setValueAt<DatasetLayerIndex>(index)({
                        dataset: vectorLayers[layer_index].dataset,
                        layer: {
                          type: "Vector",
                          index: vectorLayers[layer_index].index,
                        },
                      })
                    }
                  />
                );
              case "Raster":
                return (
                  <IndexedOptionPicker
                    prompt={param.label}
                    emptyText="There are no raster layers to select"
                    index={rasterLayers.findIndex(
                      (layer) =>
                        layer.dataset === param.value.dataset &&
                        layer.type === "Raster" &&
                        layer.index === param.value.layer.index
                    )}
                    options={rasterLayers.map((layer) => layer.dataset_file)}
                    setIndex={(index) =>
                      setValueAt<DatasetLayerIndex>(index)({
                        dataset: rasterLayers[index].dataset,
                        layer: {
                          type: "Raster",
                          index: rasterLayers[index].index,
                        },
                      })
                    }
                  />
                );
            }
          case "Option":
            console.log(param);
            return (
              <OptionPicker
                prompt={param.label}
                emptyText="No options are available to pick"
                selectedOption={param.value}
                setOption={setValueAt(index)}
                options={param.options}
              />
            );
          case "Flag":
            return (
              <Checkbox
                label={param.label}
                binding={makeParamBinding(param, index)}
              />
            );
          default:
            return <div>Got unknown type</div>;
        }
      })}
      <button
        onClick={() => {
          client.runTool(
            index,
            params.map(
              (param): ParameterValue => ({
                [param.type]: param.value,
              })
            )
          );
          setOpen(false);
        }}
      >
        Run
      </button>
    </Dialog>
  );
};

const ToolActionDialog = ({
  action,
  onClose,
}: {
  action: SavedToolOutputAction;
  onClose?: () => void;
}) => {
  const { open, setOpen, innerRef } = useDialog<HTMLHeadingElement>();
  return (
    <Dialog
      modal={true}
      openText={action.tool}
      open={!action.read || open}
      setOpen={setOpen}
      onClose={onClose}
    >
      <h1 ref={innerRef}>{action.tool}</h1>
      <div>{action.action.data}</div>
      {onClose && <button onClick={onClose}>Close</button>}
    </Dialog>
  );
};

export const ToolActionsScreen = ({
  outputs,
}: {
  outputs: SavedToolOutputAction[];
}) => {
  return (
    <div>
      <h2>Tool outputs</h2>
      <div>
        <ul>
          {outputs.map((output) => (
            <ToolActionDialog key={output.id} action={output} />
          ))}
        </ul>
      </div>{" "}
    </div>
  );
};

export const ToolOutputsPopup = () => {
  const unreadOutputs = state.value.tool_outputs.filter(
    (output) => !output.read
  );
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    if (unreadOutputs.length > 0) {
      ref?.current?.showModal();
      ref.current?.focus();
    } else {
      ref.current?.close();
    }
  }, [unreadOutputs.length]);
  return (
    <dialog ref={ref}>
      {[
        unreadOutputs.map((output) => (
          <ToolActionDialog
            key={output.id}
            action={output}
            onClose={() => client.markToolOutputRead(output.id)}
          />
        )),
      ]}
    </dialog>
  );
};
