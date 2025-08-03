import { useEffect, useRef, useState } from "preact/hooks";
import {
  Checkbox,
  Input as TextInput,
  NumberInput,
  useBindedObjectProperties,
} from "./binded-input";
import {
  DatasetLayerIndex,
  Input,
  InputType,
  LayerDescriptor,
  ParameterValue,
  SavedToolOutputAction,
  ToolDescriptor,
  ToolsScreenInfo,
  UserDefinedTool,
} from "./bindings";
import { Dialog, useDialog } from "./dialog";
import { IndexedOptionPicker, OptionPicker } from "./option-picker";
import { client, state } from "./api";
import { SaveButton } from "./save-button";

export const ToolsScreen = ({ tools, layers }: ToolsScreenInfo) => {
  return (
    <div>
      <h2>Tools</h2>
      <ToolCreationDialog />
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
  | { type: "File"; value: string }
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
        case "File":
          return {
            type: "File",
            value: "",
            label: input.label,
          };
      }
    })
  );
  const setValueAt =
    <T,>(index: number) =>
    (value: T) => {
      const replacement = { ...params[index], value };
      const newArray = params.slice();
      newArray.splice(index, 1, replacement as any);
      setParams(newArray);
    };

  const makeParamBinding = <T extends Param>(
    param: T,
    index: number
  ): { value: T["value"]; setValue: (value: T["value"]) => void } => ({
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
  const rasterLayers = layers.filter((layer) => layer.type === "Raster");

  return (
    <Dialog openText={tool.label} modal={true} open={open} setOpen={setOpen}>
      <h3>{tool.label}</h3>
      {params.map((param, index) => {
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
              <TextInput
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
          case "File":
            return (
              <SaveButton
                text={param.label + ": " + param.value}
                prompt={param.label}
                onSave={makeParamBinding(param, index).setValue}
              />
            );
          default:
            return <div>Got unknown type {JSON.stringify(param)}</div>;
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
      <h3 ref={innerRef}>{action.tool}</h3>
      <div>{action.message}</div>
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

type InputTypeDiscriminant = InputType["type"];

const inputTypeDiscriminants: InputTypeDiscriminant[] = [
  "Float",
  "Int",
  "String",
  "Dataset",
  "Layer",
  "Option",
  "Flag",
  "File",
];

const inputTypeFromDiscriminant = (
  discriminant: InputTypeDiscriminant
): InputType => {
  switch (discriminant) {
    case "Int":
    case "Float":
    case "String":
    case "Dataset":
      return { type: discriminant };
    case "Layer":
      return { type: discriminant, options: "Vector" };
    case "Option":
      return { type: discriminant, options: [] };
    case "Flag":
      return { type: discriminant };
    case "File":
      return {
        type: "File",
      };
  }
};

const ToolCreationDialog = () => {
  const tool = useBindedObjectProperties(
    ...useState<UserDefinedTool>({
      label: "",
      inputs: [],
      command: "",
      output_actions: [],
    })
  );
  const setInputAt = (index: number, input: Input) => {
    const newInputs = tool.inputs.value.slice();
    newInputs.splice(index, 1, input);
    tool.inputs.setValue(newInputs);
  };

  const { open, setOpen, innerRef } = useDialog();
  return (
    <Dialog
      openText="Create custom tool"
      open={open}
      setOpen={setOpen}
      modal={true}
    >
      <h3>New Tool</h3>
      <TextInput label="Label for tool" binding={tool.label} />
      <TextInput label="Command to run" binding={tool.command} />
      <div>
        {tool.inputs.value.map((input, index) => (
          <div>
            <TextInput
              label="label for input when running tool"
              binding={{
                value: input.label,
                setValue: (value) =>
                  setInputAt(index, { ...input, label: value }),
              }}
            />
            <button
              role="switch"
              aria-checked={input.name !== null}
              onClick={() =>
                setInputAt(index, {
                  ...input,
                  name: input.name === null ? "" : null,
                })
              }
            >
              Named
            </button>
            {input.name === null ? (
              <></>
            ) : (
              <TextInput
                label="Name to pass to command"
                binding={{
                  value: input.name,
                  setValue: (value) =>
                    setInputAt(index, { ...input, name: value }),
                }}
              />
            )}
            <OptionPicker
              options={inputTypeDiscriminants}
              selectedOption={input.param_type.type}
              setOption={(option) =>
                setInputAt(index, {
                  ...input,
                  param_type: inputTypeFromDiscriminant(option),
                })
              }
              emptyText="This should not be empty"
            />
            {input.param_type.type === "Layer" ? (
              <OptionPicker
                options={["Vector", "Raster"] as const}
                selectedOption={input.param_type.options}
                prompt="Layer type"
                emptyText="This should not be empty"
                setOption={(option) =>
                  setInputAt(index, {
                    ...input,
                    param_type: { type: "Layer", options: option },
                  })
                }
              />
            ) : input.param_type.type === "Option" ? (
              <div>
                {input.param_type.options.map(
                  (option, option_index, options) => (
                    <TextInput
                      label={"Option " + (option_index + 1)}
                      binding={{
                        value: option,
                        setValue: (option) =>
                          setInputAt(index, {
                            ...input,
                            param_type: {
                              type: "Option",
                              options: (() => {
                                const newOptions = options.slice();
                                newOptions[option_index] = option;
                                return newOptions;
                              })(),
                            },
                          }),
                      }}
                    />
                  )
                )}
                <button
                  onClick={() =>
                    setInputAt(index, {
                      ...input,
                      param_type: {
                        type: "Option",
                        options: [...input.param_type.options, ""],
                      },
                    })
                  }
                  autofocus={true}
                >
                  Add Option
                </button>
              </div>
            ) : (
              <></>
            )}
          </div>
        ))}
        <button
          onClick={() =>
            tool.inputs.setValue([
              ...tool.inputs.value,
              { label: "", name: null, param_type: { type: "String" } },
            ])
          }
        >
          Add input
        </button>
      </div>
      <div>
        {tool.output_actions.value.map((outputAction, index) => (
          <div>{outputAction}</div>
        ))}
      </div>{" "}
      <button
        onClick={() => {
          client.addCustomTool({
            label: tool.label.value,
            inputs: tool.inputs.value,
            command: tool.command.value,
            output_actions: tool.output_actions.value,
          });
          setOpen(false);
        }}
      >
        Create
      </button>
    </Dialog>
  );
};
