import { useEffect, useRef, useState } from "preact/hooks";
import {
  Checkbox,
  Input as TextInput,
  NumberInput,
  useBindedObjectProperties,
  GetSet,
} from "./binded-input";
import {
  DatasetLayerIndex,
  Input,
  InputType,
  InputTypeDiscriminants,
  LayerDescriptor,
  PresetParameterValue,
  PresetParameterValueDiscriminants,
  SavedToolOutputAction,
  ToolDescriptor,
  ToolsScreenInfo,
  UserDefinedTool,
} from "./bindings";
import { Dialog, useDialog } from "./dialog";
import {
  bindedSelectorFactory,
  IndexedOptionPicker,
  OptionPicker,
} from "./option-picker";
import { client, state } from "./api";
import { LoadButton, SaveButton } from "./save-button";

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
  | { type: "Preset"; value: PresetParameterValue }
);

const paramFromInput = (input: Input): Param => {
  switch (input.param_type.type) {
    case "Float":
      return { type: input.param_type.type, value: 0, label: input.label };
    case "Int":
      return { type: input.param_type.type, value: 0, label: input.label };
    case "String":
      return { type: input.param_type.type, value: "", label: input.label };
    case "Dataset":
      return { type: input.param_type.type, value: 0, label: input.label };
    case "Layer":
      return {
        type: input.param_type.type,
        value: {
          dataset: 0,
          layer: { type: input.param_type.options, index: 0 },
        },
        label: input.label,
        options: input.param_type.options,
      };
    case "Option":
      return {
        type: input.param_type.type,
        value: input.param_type.options[0] ?? null,
        label: input.label,
        options: input.param_type.options,
      };
    case "Flag":
      return {
        type: input.param_type.type,
        value: false,
        label: input.label,
      };
    case "File":
      return { type: input.param_type.type, value: "", label: input.label };
    case "Preset":
      return {
        type: input.param_type.type,
        value: input.param_type.options,
        label: input.label,
      };
  }
};

type ToolDialogProps = {
  tool: ToolDescriptor;
  index: number;
  layers: LayerDescriptor[];
};

const ToolDialog = ({ tool, index, layers }: ToolDialogProps) => {
  const { open, setOpen } = useDialog();

  const [params, setParams] = useState(() => tool.inputs.map(paramFromInput));

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
      {params.map((param, index) => (
        <ToolInput
          param={param}
          index={index}
          makeParamBinding={makeParamBinding}
          vectorLayers={vectorLayers}
          rasterLayers={rasterLayers}
          datasets={datasets}
          setValueAt={setValueAt}
        />
      ))}
      <button
        onClick={() => {
          client.runTool(
            index,
            params.filter((param) => param.type !== "Preset")
          );
          setOpen(false);
        }}
      >
        Run
      </button>
    </Dialog>
  );
};

const ToolInput = ({
  param,
  index,
  makeParamBinding,
  datasets,
  setValueAt,
  vectorLayers,
  rasterLayers,
}: {
  param: Param;
  index: number;
  makeParamBinding: <T extends Param>(
    param: T,
    index: number
  ) => GetSet<T["value"]>;
  datasets: string[];
  vectorLayers: LayerDescriptor[];
  rasterLayers: LayerDescriptor[];
  setValueAt: <T>(index: number) => (value: T) => void;
}) => {
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
          label={param.label}
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
        <div>
          <SaveButton
            text={param.label + ": " + param.value}
            prompt={param.label}
            onSave={makeParamBinding(param, index).setValue}
          />
        </div>
      );
    case "Preset":
      return (
        <div>
          {param.label}: {param.value.value}
        </div>
      );
    default:
      return <div>Got unknown type {JSON.stringify(param)}</div>;
  }
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
      <div>{action.output}</div>
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
      {unreadOutputs.map((output) => (
        <ToolActionDialog
          key={output.id}
          action={output}
          onClose={() => client.markToolOutputRead(output.id)}
        />
      ))}
    </dialog>
  );
};

const InputTypeDiscriminantSelector = bindedSelectorFactory(
  await client.getToolInputTypes()
);

const inputTypeFromDiscriminant = (
  discriminant: InputTypeDiscriminants
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
      return { type: discriminant, options: false };
    case "Preset":
      return { type: discriminant, options: { type: "File", value: "" } };
  }
};

const PresetInputTypeSelector = bindedSelectorFactory(
  await client.getToolPresetInputTypes()
);

const presetInputFromDiscriminant = (
  discriminant: PresetParameterValueDiscriminants
): PresetParameterValue => {
  switch (discriminant) {
    case "Float":
    case "Int":
      return { type: discriminant, value: 0 };
    case "String":
    case "File":
      return { type: discriminant, value: "" };
  }
};

const ToolCreationDialog = () => {
  const tool = useBindedObjectProperties(
    ...useState<UserDefinedTool>({
      label: "",
      inputs: [],
      command: "",
      output_actions: { alert_output: true, load_layers: [] },
    })
  );

  const setInputAt = (index: number, element: Input) => {
    const newArray = tool.inputs.value.slice();
    newArray.splice(index, 1, element);
    tool.inputs.setValue(newArray);
  };

  const { open, setOpen } = useDialog();

  const output_files = tool.inputs.value
    .filter((input) => input.param_type.type === "File")
    .map((input) => input.label);

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
          <ToolInputCreator
            input={input}
            setInput={(input) => setInputAt(index, input)}
          />
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
        <button
          role="switch"
          aria-checked={tool.output_actions.value.alert_output}
          onClick={() =>
            tool.output_actions.setValue({
              alert_output: !tool.output_actions.value.alert_output,
              load_layers: tool.output_actions.value.load_layers,
            })
          }
        >
          Alert output when ran?
        </button>
        <div>
          {tool.output_actions.value.load_layers.map((layer, index) => (
            <IndexedOptionPicker
              prompt="File to load"
              index={layer}
              options={output_files}
              setIndex={(layer) => {
                let newArray = tool.output_actions.value.load_layers.slice();
                newArray[index] = layer;
                tool.output_actions.setValue({
                  alert_output: tool.output_actions.value.alert_output,
                  load_layers: newArray,
                });
              }}
              emptyText="There are no files marked as use as output"
            />
          ))}
          <button
            onClick={() =>
              tool.output_actions.setValue({
                alert_output: tool.output_actions.value.alert_output,
                load_layers: [
                  ...tool.output_actions.value.load_layers,
                  output_files.length - 1,
                ],
              })
            }
          >
            Add layer to load
          </button>{" "}
        </div>
      </div>
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

const ToolInputCreator = ({
  input,
  setInput,
}: {
  input: Input;
  setInput: (element: Input) => void;
}) => (
  <div>
    <TextInput
      label="label for input when running tool"
      binding={{
        value: input.label,
        setValue: (value) => setInput({ ...input, label: value }),
      }}
    />
    <button
      role="switch"
      aria-checked={input.name !== null}
      onClick={() =>
        setInput({
          ...input,
          name: input.name === null ? "" : null,
        })
      }
    >
      Named
    </button>
    {input.name === null ? null : (
      <TextInput
        label="Name to pass to command"
        binding={{
          value: input.name,
          setValue: (value) => setInput({ ...input, name: value }),
        }}
      />
    )}
    <InputTypeDiscriminantSelector
      prompt="Type of input"
      binding={{
        value: input.param_type.type,
        setValue: (option) =>
          setInput({
            ...input,
            param_type: inputTypeFromDiscriminant(option),
          }),
      }}
    />
    {input.param_type.type === "Layer" ? (
      <OptionPicker
        options={["Vector", "Raster"] as const}
        selectedOption={input.param_type.options}
        prompt="Layer type"
        emptyText="This should not be empty"
        setOption={(option) =>
          setInput({
            ...input,
            param_type: { type: "Layer", options: option },
          })
        }
      />
    ) : input.param_type.type === "Option" ? (
      <div>
        {input.param_type.options.map((option, option_index, options) => (
          <TextInput
            label={"Option " + (option_index + 1)}
            binding={{
              value: option,
              setValue: (option) =>
                setInput({
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
        ))}
        <button
          onClick={() => {
            // Not really needed but there is the odd edge case and makes ts happy
            if (input.param_type.type === "Option")
              setInput({
                ...input,
                param_type: {
                  type: "Option",
                  options: [...input.param_type.options, ""],
                },
              });
          }}
          autofocus={true}
        >
          Add Option
        </button>
      </div>
    ) : input.param_type.type === "Preset" ? (
      <>
        <PresetInputTypeSelector
          prompt="Type of preset input"
          binding={{
            value: input.param_type.options.type,
            setValue: (option) =>
              setInput({
                ...input,
                param_type: {
                  type: "Preset",
                  options: presetInputFromDiscriminant(option),
                },
              }),
          }}
        />
        <PresetInputValueEditor
          input={input.param_type.options}
          setInput={(value) =>
            setInput({
              ...input,
              param_type: { type: "Preset", options: value },
            })
          }
        />
      </>
    ) : input.param_type.type === "File" ? (
      <button
        role="switch"
        aria-checked={input.param_type.options}
        onClick={() => {
          // Here to make TS happy and just in case of the rare edge case
          if (input.param_type.type === "File") {
            setInput({
              ...input,
              param_type: { type: "File", options: !input.param_type.options },
            });
          }
        }}
      >
        Use as output
      </button>
    ) : null}
  </div>
);

type PresetInputValueEditorProps = {
  input: PresetParameterValue;
  setInput: (input: PresetParameterValue) => void;
};

const PresetInputValueEditor = ({
  input,
  setInput,
}: PresetInputValueEditorProps) => {
  switch (input.type) {
    case "Float":
    case "Int":
      return (
        <NumberInput
          label="Preset value"
          binding={{
            value: input.value,
            setValue: (value) => setInput({ type: input.type, value }),
          }}
        />
      );
    case "String":
      return (
        <TextInput
          label="Preset value"
          binding={{
            value: input.value,
            setValue: (value) => setInput({ type: input.type, value }),
          }}
        />
      );
    case "File":
      return (
        <LoadButton
          text={"Preset input: " + input.value}
          onLoad={(value) => setInput({ type: input.type, value })}
        />
      );
  }
};
