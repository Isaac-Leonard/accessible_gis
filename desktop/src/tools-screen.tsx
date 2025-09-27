import { useEffect, useRef, useState } from "preact/hooks";
import {
  Checkbox,
  Input as TextInput,
  NumberInput,
  propertyGetSet,
  GetSet,
  useBindedObjectProperties,
} from "./binded-input";
import {
  DatasetLayerIndex,
  ToolInputType,
  ToolInputTypeDiscriminants,
  LayerDescriptor,
  NewToolInput,
  NewUserDefinedTool,
  ToolParameterValue,
  ToolPresetParameterValue,
  ToolPresetParameterValueDiscriminants,
  SavedToolOutputAction,
  ToolDescriptor,
  ToolOutput,
  ToolParameter,
  ToolsScreenInfo,
  ToolRuntimeInputDescriptor,
  LayerType,
  ToolFileInput,
} from "./bindings";
import { Dialog, useDialog } from "./dialog";
import {
  bindedSelectorFactory,
  IndexedOptionPicker,
  OptionPicker,
} from "./option-picker";
import { client, state } from "./api";
import { LoadButton, SaveButton } from "./save-button";
import { JSX } from "preact/jsx-runtime";

export const ToolsScreen = ({ tools, layers }: ToolsScreenInfo) => {
  return (
    <div>
      <h2>Tools</h2>
      <ToolsActionsDialog tools={tools} />
      <div>
        {tools.map((tool) => (
          <ToolDialog
            key={tool.id}
            tool={tool}
            layers={layers}
            run={client.runTool}
          />
        ))}
      </div>
    </div>
  );
};

type ParameterDescriptor = { label: string; id: string } & (
  | { optional: true; included: boolean }
  | { optional: false }
) &
  (
    | { type: "Float"; value: number }
    | { type: "Int"; value: number }
    | { type: "String"; value: string }
    | { type: "Dataset"; value: number }
    | { type: "Layer"; value: DatasetLayerIndex; options: LayerType }
    | { type: "Option"; value: string; options: string[] }
    | { type: "File"; value: ToolFileInput; output: boolean }
  );

export const toolInputParamFromInput = (
  input: ToolRuntimeInputDescriptor
): ParameterDescriptor => {
  const optional = input.optional
    ? { optional: input.optional, included: false }
    : { optional: input.optional };

  switch (input.param_type.type) {
    case "Float":
      return {
        type: input.param_type.type,
        value: 0,
        label: input.label,
        id: input.id,
        ...optional,
      };
    case "Int":
      return {
        type: input.param_type.type,
        value: 0,
        label: input.label,
        id: input.id,
        ...optional,
      };
    case "String":
      return {
        type: input.param_type.type,
        value: "",
        label: input.label,
        id: input.id,
        ...optional,
      };
    case "Dataset":
      return {
        type: input.param_type.type,
        value: 0,
        label: input.label,
        id: input.id,
        ...optional,
      };
    case "Layer":
      return {
        type: input.param_type.type,
        value: {
          dataset: 0,
          layer: {
            type:
              input.param_type.options === "Any"
                ? "Vector"
                : input.param_type.options,
            index: 0,
          },
        },
        label: input.label,
        options: input.param_type.options,
        id: input.id,
        ...optional,
      };
    case "Option":
      return {
        type: input.param_type.type,
        value: input.param_type.options[0] ?? null,
        label: input.label,
        options: input.param_type.options,
        id: input.id,
        ...optional,
      };
    case "File":
      return {
        type: input.param_type.type,
        value: { type: "Named", value: "" },
        label: input.label,
        id: input.id,
        ...optional,
        output: input.param_type.options,
      };
  }
};

type ToolDialogProps = {
  tool: ToolDescriptor;
  layers: LayerDescriptor[];
  run: (id: string, params: ToolParameter[]) => void;
};

export const ToolDialog = ({ tool, layers, run }: ToolDialogProps) => {
  const { open, setOpen } = useDialog();

  const [params, setParams] = useState(() =>
    tool.inputs.map(toolInputParamFromInput)
  );

  const setIncludeParam = (index: number, included: boolean) => {
    const replacement = { ...params[index], included };
    const newArray = params.slice();
    newArray[index] = replacement;
    setParams(newArray);
  };

  const makeParamBinding = (param: ParameterDescriptor, index: number) => ({
    value: param,
    setValue: (value: ParameterDescriptor) => {
      const newParams = params.slice();
      newParams[index] = value;
      setParams(newParams);
    },
  });

  const datasets = [...new Set(layers.map((ds) => ds.dataset_file))];

  const vectorLayers = layers.filter((layer) => layer.type === "Vector");
  const rasterLayers = layers.filter((layer) => layer.type === "Raster");

  return (
    <Dialog openText={tool.label} modal={true} open={open} setOpen={setOpen}>
      <h3>{tool.label}</h3>
      {params.map((param, index) => (
        <div>
          {param.optional ? (
            <button
              role="switch"
              aria-checked={param.included}
              onClick={() => setIncludeParam(index, !param.included)}
            >
              Include {param.label}
            </button>
          ) : null}
          {(param.optional && param.included) || !param.optional ? (
            <ToolInput
              parameterBinding={makeParamBinding(param, index)}
              vectorLayers={vectorLayers}
              rasterLayers={rasterLayers}
              datasets={datasets}
            />
          ) : null}
        </div>
      ))}
      <button
        onClick={() => {
          run(tool.id, parametersFromUi(params));
          setOpen(false);
        }}
      >
        Run
      </button>
    </Dialog>
  );
};

const parametersFromUi = (params: ParameterDescriptor[]): ToolParameter[] => {
  return params
    .filter((param) => !(param.optional && !param.included))
    .map((param) => ({
      id: param.id,
      value: toolInputFromDescriptor(param),
    }));
};

// TODO: Do proper checking for validity here
const toolInputFromDescriptor = (
  param: ParameterDescriptor
): ToolParameterValue =>
  ({ type: param.type, value: param.value } as ToolParameterValue);

const ToolInput = ({
  parameterBinding,
  datasets,
  vectorLayers,
  rasterLayers,
}: {
  parameterBinding: GetSet<ParameterDescriptor>;
  datasets: string[];
  vectorLayers: LayerDescriptor[];
  rasterLayers: LayerDescriptor[];
}): JSX.Element => {
  switch (parameterBinding.value.type) {
    case "Float":
      return (
        <NumberInput
          label={parameterBinding.value.label}
          binding={propertyGetSet("value", parameterBinding)}
        />
      );
    case "Int":
      return (
        <NumberInput
          label={parameterBinding.value.label}
          binding={propertyGetSet("value", parameterBinding)}
        />
      );
    case "String":
      return (
        <TextInput
          label={parameterBinding.value.label}
          binding={propertyGetSet("value", parameterBinding)}
        />
      );
    case "Dataset":
      return (
        <IndexedOptionPicker
          prompt={parameterBinding.value.label}
          emptyText="No datasets to select"
          index={parameterBinding.value.value}
          options={datasets}
          setIndex={(value) =>
            parameterBinding.setValue({
              ...parameterBinding.value,
              type: "Dataset",
              value,
            })
          }
        />
      );
    case "Layer":
      switch (parameterBinding.value.options) {
        case "Vector":
          return (
            <IndexedOptionPicker
              prompt={parameterBinding.value.label}
              emptyText="There are no vector layers to select"
              index={vectorLayers.findIndex(
                (layer) =>
                  // First check just to satisfy type script
                  parameterBinding.value.type === "Layer" &&
                  layer.dataset === parameterBinding.value.value.dataset &&
                  layer.type === "Vector" &&
                  layer.index === parameterBinding.value.value.layer.index
              )}
              options={vectorLayers.map((layer) => layer.dataset_file)}
              setIndex={(layer_index) =>
                propertyGetSet("value", parameterBinding).setValue({
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
              prompt={parameterBinding.value.label}
              emptyText="There are no raster layers to select"
              index={rasterLayers.findIndex(
                (layer) =>
                  // First check just to satisfy type script
                  parameterBinding.value.type === "Layer" &&
                  layer.dataset === parameterBinding.value.value.dataset &&
                  layer.type === "Raster" &&
                  layer.index === parameterBinding.value.value.layer.index
              )}
              options={rasterLayers.map((layer) => layer.dataset_file)}
              setIndex={(layer_index) =>
                propertyGetSet("value", parameterBinding).setValue({
                  dataset: rasterLayers[layer_index].dataset,
                  layer: {
                    type: "Raster",
                    index: rasterLayers[layer_index].index,
                  },
                })
              }
            />
          );
        case "Any":
          return (
            <div>
              <button
                onClick={() =>
                  parameterBinding.setValue({
                    ...parameterBinding.value,
                    type: "Layer",
                    options: "Any",
                    value: {
                      dataset: 0,
                      layer: {
                        type:
                          (parameterBinding.value.value as DatasetLayerIndex)
                            .layer.type === "Vector"
                            ? "Raster"
                            : "Vector",
                        index:
                          (parameterBinding.value.value as DatasetLayerIndex)
                            .layer.type === "Vector"
                            ? 0
                            : 1,
                      },
                    },
                  })
                }
              >
                {parameterBinding.value.value.layer.type === "Vector"}
              </button>
              {parameterBinding.value.value.layer.type === "Vector" ? (
                <IndexedOptionPicker
                  prompt={parameterBinding.value.label}
                  emptyText="There are no vector layers to select"
                  index={vectorLayers.findIndex(
                    (layer) =>
                      // First check just to satisfy type script
                      parameterBinding.value.type === "Layer" &&
                      layer.dataset === parameterBinding.value.value.dataset &&
                      layer.type === "Vector" &&
                      layer.index === parameterBinding.value.value.layer.index
                  )}
                  options={vectorLayers.map((layer) => layer.dataset_file)}
                  setIndex={(layer_index) =>
                    propertyGetSet("value", parameterBinding).setValue({
                      dataset: vectorLayers[layer_index].dataset,
                      layer: {
                        type: "Vector",
                        index: vectorLayers[layer_index].index,
                      },
                    })
                  }
                />
              ) : (
                <IndexedOptionPicker
                  prompt={parameterBinding.value.label}
                  emptyText="There are no raster layers to select"
                  index={rasterLayers.findIndex(
                    (layer) =>
                      // First check just to satisfy type script
                      parameterBinding.value.type === "Layer" &&
                      layer.dataset === parameterBinding.value.value.dataset &&
                      layer.type === "Raster" &&
                      layer.index === parameterBinding.value.value.layer.index
                  )}
                  options={rasterLayers.map((layer) => layer.dataset_file)}
                  setIndex={(layer_index) =>
                    propertyGetSet("value", parameterBinding).setValue({
                      dataset: rasterLayers[layer_index].dataset,
                      layer: {
                        type: "Raster",
                        index: rasterLayers[layer_index].index,
                      },
                    })
                  }
                />
              )}
            </div>
          );
      }
    case "Option":
      return (
        <OptionPicker
          prompt={parameterBinding.value.label}
          emptyText="No options are available to pick"
          selectedOption={parameterBinding.value.value}
          setOption={propertyGetSet("value", parameterBinding).setValue}
          options={parameterBinding.value.options}
        />
      );
    case "File":
      return (
        <FileInput
          label={`${parameterBinding.value.label}: ${
            parameterBinding.value.value.value ?? ""
          }`}
          file={parameterBinding.value.value}
          setFile={(file) =>
            parameterBinding.setValue({
              ...parameterBinding.value,
              type: "File",
              value: file,
              output: (parameterBinding.value as { output: boolean }).output,
            })
          }
          output={parameterBinding.value.output}
        />
      );
    default:
      return (
        <div>Got unknown type {JSON.stringify(parameterBinding.value)}</div>
      );
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
      <ToolOutputViewer output={action.output} />
      {onClose && <button onClick={onClose}>Close</button>}
    </Dialog>
  );
};

const ToolOutputViewer = ({ output }: { output: ToolOutput }) => {
  switch (output.returned_output?.type) {
    case null:
      return <div>No output</div>;
    case "Command":
      return (
        <div>
          Status: {output.returned_output.value.status}
          <h4>Stderr:</h4>
          {output.returned_output.value.stderr}
          <h4>Stdout</h4>
          {output.returned_output.value.stdout}{" "}
        </div>
      );
    case "String":
      return <div>{output.returned_output.value}</div>;
    default:
      return <div>Unknown output type {output.returned_output?.type}</div>;
  }
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

export const ToolInputTypeDiscriminantSelector = bindedSelectorFactory(
  await client.getToolInputTypes()
);

export const toolInputTypeFromDiscriminant = (
  discriminant: ToolInputTypeDiscriminants
): ToolInputType => {
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
    case "File":
      return { type: discriminant, options: false };
  }
};

export const PresetInputTypeSelector = bindedSelectorFactory(
  await client.getToolPresetInputTypes()
);

export const presetInputFromDiscriminant = (
  discriminant: ToolPresetParameterValueDiscriminants
): ToolPresetParameterValue => {
  switch (discriminant) {
    case "Float":
    case "Int":
      return { type: discriminant, value: 0 };
    case "String":
      return { type: discriminant, value: "" };
    case "File":
      return { type: discriminant, value: { type: "Named", value: "" } };
  }
};

const ToolCreationDialog = () => {
  const tool = useBindedObjectProperties(
    ...useState<NewUserDefinedTool>({
      label: "",
      inputs: [],
      command: "",
      output_actions: { alert_output: true, load_layers: [] },
    })
  );

  const setInputAt = (index: number, element: NewToolInput) => {
    const newArray = tool.inputs.value.slice();
    newArray.splice(index, 1, element);
    tool.inputs.setValue(newArray);
  };

  const { open, setOpen } = useDialog();

  const output_files = tool.inputs.value
    .filter((input) => input.type === "Runtime")
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
              {
                type: "Runtime",
                label: "",
                param_type: { type: "String" },
                optional: false,
              },
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
  input: NewToolInput;
  setInput: (element: NewToolInput) => void;
}) => {
  return (
    <div>
      <button
        role="switch"
        aria-checked={input.type === "Preset"}
        onClick={() =>
          setInput(
            input.type === "Preset"
              ? {
                  type: "Runtime",
                  label: "",
                  optional: false,
                  param_type: { type: "String" },
                }
              : { type: "Preset", value: { type: "String", value: "" } }
          )
        }
      >
        Preset value
      </button>
      {input.type === "Preset" ? (
        <>
          <PresetInputTypeSelector
            prompt="Type of preset input"
            binding={{
              value: input.value.type,
              setValue: (discriminant) =>
                setInput({
                  type: "Preset",
                  value: presetInputFromDiscriminant(discriminant),
                }),
            }}
          />
          <PresetInputValueEditor
            input={input.value}
            setInput={(value) => setInput({ type: "Preset", value })}
          />
        </>
      ) : (
        <div>
          <TextInput
            label="label for input when running tool"
            binding={{
              value: input.label,
              setValue: (value) => setInput({ ...input, label: value }),
            }}
          />
          <Checkbox
            label="Optional"
            binding={{
              value: input.optional,
              setValue: (value) => setInput({ ...input, optional: value }),
            }}
          />
          <ToolInputTypeDiscriminantSelector
            prompt="Type of input"
            binding={{
              value: input.param_type.type,
              setValue: (option) =>
                setInput({
                  ...input,
                  param_type: toolInputTypeFromDiscriminant(option),
                }),
            }}
          />
          {input.param_type.type === "Layer" ? (
            <OptionPicker
              options={["Vector", "Raster", "Any"] as const}
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
          ) : input.param_type.type === "File" ? (
            <button
              role="switch"
              aria-checked={input.param_type.options}
              onClick={() => {
                // Here to make TS happy and just in case of the rare edge case
                if (input.param_type.type === "File") {
                  setInput({
                    ...input,
                    param_type: {
                      type: "File",
                      options: !input.param_type.options,
                    },
                  });
                }
              }}
            >
              Use as output
            </button>
          ) : null}
        </div>
      )}
    </div>
  );
};

type PresetInputValueEditorProps = {
  input: ToolPresetParameterValue;
  setInput: (input: ToolPresetParameterValue) => void;
};

export const PresetInputValueEditor = ({
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
        <FileInput
          label={`Preset value: ${input.value.value ?? ""}`}
          file={input.value}
          setFile={(file) => setInput({ type: "File", value: file })}
          output={false}
        />
      );
  }
};

type FileInputProps = {
  label: String;
  file: ToolFileInput;
  setFile: (file: ToolFileInput) => void;
  output: boolean;
};

const FileInput = ({ label, file, setFile, output }: FileInputProps) => (
  <div>
    {label}
    <button
      onClick={() =>
        setFile(
          file.type === "Temp"
            ? { type: "Named", value: "" }
            : { type: "Temp", value: null }
        )
      }
    >
      {file.type}
    </button>
    {file.type === "Named" ? (
      output ? (
        <SaveButton
          text={"Preset input: " + file.value}
          onSave={(value) => setFile({ type: "Named", value })}
        />
      ) : (
        <LoadButton
          text={"Preset input: " + file.value}
          onLoad={(value) => setFile({ type: "Named", value })}
        />
      )
    ) : (
      <div>
        <button
          role="switch"
          aria-checked={file.value !== null}
          onClick={() =>
            setFile({ type: "Temp", value: file.value === null ? "" : null })
          }
        >
          Extention
        </button>
        {file.value !== null ? (
          <TextInput
            label="Extention"
            binding={{
              value: file.value,
              setValue: (value) => setFile({ type: "Temp", value }),
            }}
          />
        ) : null}
      </div>
    )}
  </div>
);

const ToolsActionsDialog = ({ tools }: { tools: ToolDescriptor[] }) => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      modal={true}
      openText="Actions for tools"
      open={open}
      setOpen={setOpen}
    >
      <h4>Tool Actions</h4>
      <ToolCreationDialog />
      <BulkSaveDialog tools={tools} />
      <LoadButton text="Load tools" onLoad={client.loadToolsBulk} />
    </Dialog>
  );
};

const BulkSaveDialog = ({ tools }: { tools: ToolDescriptor[] }) => {
  const [toolsToSave, setToolsToSave] = useState<string[]>([]);
  const { open, setOpen } = useDialog();
  return (
    <Dialog modal={true} open={open} setOpen={setOpen} openText="Bulk save">
      <h5>Bulk Save Tools</h5>
      <div>Select tools to save:</div>
      <div>
        {tools.map((tool) => (
          <Checkbox
            key={tool.id}
            label={tool.label}
            binding={{
              value: toolsToSave.includes(tool.id),
              setValue: (include) =>
                include
                  ? setToolsToSave([...toolsToSave, tool.id])
                  : setToolsToSave(toolsToSave.filter((id) => tool.id !== id)),
            }}
          />
        ))}
      </div>
      <SaveButton
        text="Save to file"
        onSave={(file) => {
          console.log("Saving to " + file);
          console.log(toolsToSave);
          client.saveToolsBulk(toolsToSave, file);
          setOpen(false);
        }}
      />
    </Dialog>
  );
};
