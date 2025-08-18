import { useState } from "preact/hooks";
import {
  ToolDescriptor,
  Workflow,
  NewWorkflowInputDescriptor,
  WorkflowInputValueDescriptor,
  WorkflowInputValueDescriptorDiscriminants,
  DatasetLayerIndex,
  WorkflowInputDescriptor,
  LayerIndexDiscriminants,
  LayerDescriptor,
  WorkflowInputValue,
  WorkflowsScreenInfo,
} from "./bindings";
import {
  bindedSelectorFactory,
  IndexedOptionPicker,
  OptionPicker,
} from "./option-picker";
import { Dialog, useDialog } from "./dialog";
import { client } from "./api";
import { Checkbox, NumberInput, Input as TextInput } from "./binded-input";
import { SaveButton } from "./save-button";
import { ReactElement } from "preact/compat";

type UiWorkflowInput = { id: string; label: string } & (
  | { type: "Float"; value: number }
  | { type: "Int"; value: number }
  | { type: "String"; value: string }
  | {
      type: "Layer";
      layerType: LayerIndexDiscriminants;
      value: DatasetLayerIndex;
    }
  | {
      type: "File";
      value: { type: "Temp" } | { type: "Custom"; value: string | null };
    }
  | { type: "Option"; options: string[]; value: string }
  | { type: "Flag"; value: boolean }
);

const getWorkflowInput = (input: WorkflowInputDescriptor): UiWorkflowInput => {
  switch (input.value.type) {
    case "Float":
    case "Int":
      return {
        type: input.value.type,
        value: 0,
        label: input.label,
        id: input.id,
      };
    case "String":
      return {
        type: input.value.type,
        value: "",
        label: input.label,
        id: input.id,
      };
    case "Layer":
      return {
        type: input.value.type,
        value: {
          dataset_index: 0,
          layer: { type: input.value.value, index: null },
        },
      };
    case "File":
      return {
        type: input.value.type,
        value: { type: "Temp" },
        label: input.label,
        id: input.id,
      };
    case "Option":
      return {
        type: input.value.type,
        options: input.value.value,
        value: input.value.value[0],
        label: input.label,
        id: input.id,
      };
    case "Flag":
      return {
        type: input.value.type,
        value: false,
        label: input.label,
        id: input.id,
      };
  }
};

type RunWorkflowScreenProps = {
  workflow: Workflow;
  layers: LayerDescriptor[];
};

const RunWorkflowScreen = ({ workflow, layers }: RunWorkflowScreenProps) => {
  const [inputs, setInputs] = useState(workflow.inputs.map(getWorkflowInput));
  const setInputAt =
    <T extends UiWorkflowInput>(index: number) =>
    (value: T["value"]) => {
      const replacement = { ...(inputs[index] as T), value };
      const newInputs = inputs.slice();
      newInputs[index] = replacement;
      setInputs(newInputs);
    };

  const datasets = layers.reduce((arr, el) => {
    if (arr.includes(el.dataset_file)) {
      return arr;
    } else {
      return [...arr, el.dataset_file];
    }
  }, [] as string[]);

  const vectorLayers = layers.filter((layer) => layer.type === "Vector");
  const rasterLayers = layers.filter((layer) => layer.type === "Raster");
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      open={open}
      setOpen={setOpen}
      modal={true}
      openText={workflow.label}
    >
      {inputs.map((input, index) => (
        <WorkflowInputEditor
          input={input}
          setInputValue={setInputAt<typeof input>(index)}
          datasets={datasets}
          vectorLayers={vectorLayers}
          rasterLayers={rasterLayers}
        />
      ))}
      <button
        onClick={() =>
          client.runWorkflow({
            inputs: inputs.map((input) => ({
              id: input.id,
              value: {
                type: input.type,
                value: input.value,
              } as WorkflowInputValue,
            })),
          })
        }
      >
        Run workflow
      </button>
    </Dialog>
  );
};

type WorkflowInputEditorProps<T extends UiWorkflowInput> = {
  input: T;
  setInputValue: <T extends UiWorkflowInput>(value: T["value"]) => void;
  datasets: string[];
  vectorLayers: LayerDescriptor[];
  rasterLayers: LayerDescriptor[];
};

const WorkflowInputEditor = <T extends UiWorkflowInput>({
  input,
  setInputValue,
  vectorLayers,
  rasterLayers,
}: WorkflowInputEditorProps<T>): ReactElement => {
  switch (input.type) {
    case "Float":
    case "Int":
      return (
        <NumberInput
          label={input.label}
          binding={{
            value: input.value,
            setValue: setInputValue,
          }}
        />
      );
    case "String":
      return (
        <TextInput
          label={input.label}
          binding={{
            value: input.value,
            setValue: setInputValue<typeof input>,
          }}
        />
      );
    case "File":
      return (
        <div>
          <OptionPicker
            prompt="File"
            emptyText="This should not be empty"
            options={["Temp", "Custom"] as const}
            selectedOption={input.value.type}
            setOption={(option) =>
              setInputValue(
                option === "Temp"
                  ? { type: option }
                  : { type: option, value: null }
              )
            }
          />
          {input.value.type === "Custom" ? (
            <SaveButton
              prompt="File path"
              onSave={(file) => setInputValue({ type: "Custom", value: file })}
            />
          ) : null}{" "}
        </div>
      );
    case "Layer":
      switch (input.layerType) {
        case "Vector":
          return (
            <IndexedOptionPicker
              prompt={input.label}
              emptyText="There are no vector layers to select"
              index={vectorLayers.findIndex(
                (layer) =>
                  layer.dataset === input.value.dataset &&
                  layer.type === "Vector" &&
                  layer.index === input.value.layer.index
              )}
              options={vectorLayers.map((layer) => layer.dataset_file)}
              setIndex={(layer_index) =>
                setInputValue({
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
              prompt={input.label}
              emptyText="There are no raster layers to select"
              index={rasterLayers.findIndex(
                (layer) =>
                  layer.dataset === input.value.dataset &&
                  layer.type === "Raster" &&
                  layer.index === input.value.layer.index
              )}
              options={rasterLayers.map((layer) => layer.dataset_file)}
              setIndex={(layer_index) =>
                setInputValue({
                  dataset: rasterLayers[layer_index].dataset,
                  layer: {
                    type: "Raster",
                    index: rasterLayers[layer_index].index,
                  },
                })
              }
            />
          );
      }
    case "Option":
      return (
        <OptionPicker
          prompt={input.label}
          emptyText="No options are available to pick"
          selectedOption={input.value}
          setOption={setInputValue}
          options={input.options}
        />
      );
    case "Flag":
      return (
        <Checkbox
          label={input.label}
          binding={{ value: input.value, setValue: setInputValue }}
        />
      );
  }
};

const WorkflowInputTypeDiscriminantSelector = bindedSelectorFactory(
  await client.getWorkflowInputTypes()
);

type Connection = { tool: string; parameter: string };

const AddWorkflowScreen = ({ toolList }: { toolList: ToolDescriptor[] }) => {
  const [label, setLabel] = useState("");
  const [inputs, setInputs] = useState<NewWorkflowInputDescriptor[]>([]);
  const [tools, setTools] = useState<string[]>([]);
  const { open, setOpen } = useDialog();
  return (
    <Dialog open={open} setOpen={setOpen} modal={true} openText="Add workflow">
      <h3>Workflow creation dialog</h3>
      <TextInput
        label="Display name for workflow"
        binding={{ value: label, setValue: setLabel }}
      />
      <ListOfInputs
        tools={toolList.filter((tool) => tools.includes(tool.id))}
        inputs={inputs}
        setInputs={setInputs}
      />
      <button
        onClick={() =>
          setInputs([
            ...inputs,
            { label: "", value: { type: "Int" }, connections: [] },
          ])
        }
      >
        Add input
      </button>
      <ul>
        {tools.map((id) => (
          <li key={id}>{toolList.find((tool) => tool.id === id)?.label}</li>
        ))}
      </ul>
      <AddTool
        toolList={toolList}
        addTool={(tool) => setTools([...tools, tool])}
      />
      <button
        onClick={() => {
          client.addWorkflow({ label, tools, inputs });
        }}
      >
        Create workflow{" "}
      </button>
    </Dialog>
  );
};

const AddTool = ({
  toolList,
  addTool,
}: {
  toolList: ToolDescriptor[];
  addTool: (tool: string) => void;
}) => {
  const [tool, setTool] = useState<number | null>(null);
  const { open, setOpen } = useDialog();
  return (
    <Dialog modal={true} open={open} setOpen={setOpen} openText="Add tool">
      <IndexedOptionPicker
        prompt="Tool to add"
        emptyText="No tools added"
        index={tool}
        options={toolList.map((tool) => tool.label)}
        setIndex={setTool}
      />
      <button
        disabled={tool === null}
        onClick={() => {
          if (tool !== null) {
            setOpen(false);
            addTool(toolList[tool].id);
          }
        }}
      >
        Add
      </button>
    </Dialog>
  );
};

const getWorkflowInputTypeFromDescriminant = (
  discriminant: WorkflowInputValueDescriptorDiscriminants
): WorkflowInputValueDescriptor => {
  switch (discriminant) {
    case "Float":
    case "Int":
    case "String":
    case "File":
    case "Flag":
      return { type: discriminant };
    case "Layer":
      return { type: discriminant, value: "Vector" };
    case "Option":
      return { type: discriminant, value: [] };
  }
};

type WorkflowInputTypePickerProps = {
  type: WorkflowInputValueDescriptor;
  setType: (type: WorkflowInputValueDescriptor) => void;
};

const WorkflowInputTypePicker = ({
  type,
  setType,
}: WorkflowInputTypePickerProps) => {
  return (
    <div>
      <WorkflowInputTypeDiscriminantSelector
        prompt="Input type"
        binding={{
          value: type.type,
          setValue: (discriminant) =>
            setType(getWorkflowInputTypeFromDescriminant(discriminant)),
        }}
      />
    </div>
  );
};

const WorkflowInputDescriptorEditor = ({
  tools,
  input,
  setInput,
}: {
  input: NewWorkflowInputDescriptor;
  setInput: (input: NewWorkflowInputDescriptor) => void;
  tools: ToolDescriptor[];
}) => {
  const addConnection = (connection: Connection) =>
    setInput({
      label: input.label,
      value: input.value,
      connections: [...input.connections, connection],
    });

  const setType = (value: NewWorkflowInputDescriptor["value"]) =>
    setInput({ label: input.label, value, connections: input.connections });

  const setLabel = (label: string) =>
    setInput({ label, value: input.value, connections: input.connections });

  return (
    <div>
      <TextInput
        label="Label"
        binding={{ value: input.label, setValue: setLabel }}
      />
      <WorkflowInputTypePicker type={input.value} setType={setType} />
      <div>
        Inputs to:
        <ul>
          {input.connections.map((connection) => {
            const tool = tools.find((tool) => tool.id === connection.tool);
            const param = tool?.inputs.find(
              (input) => input.id === connection.parameter
            );
            return (
              <li key={connection.tool + ":" + connection.parameter}>
                {tool?.label}: {param?.label}
              </li>
            );
          })}
        </ul>
      </div>
      <WorkflowInputConnector tools={tools} addConnection={addConnection} />
    </div>
  );
};

const WorkflowInputConnector = ({
  tools,
  addConnection,
}: {
  tools: ToolDescriptor[];
  addConnection: (connection: Connection) => void;
}) => {
  const [tool, setTool] = useState<number | null>(null);
  const [parameter, setParameter] = useState<number | null>(null);
  return (
    <div>
      <IndexedOptionPicker
        prompt="Tool"
        emptyText="No tools added"
        options={tools.map((tool) => tool.label)}
        index={tool}
        setIndex={(index) => {
          setTool(index);
          setParameter(0);
        }}
      />
      <button
        disabled={tool === null || parameter === null}
        onClick={() => {
          if (tool !== null && parameter !== null) {
            addConnection({
              tool: tools[tool].id,
              parameter: tools[tool].inputs[parameter].id,
            });
          }
        }}
      >
        Add
      </button>
    </div>
  );
};

const ListOfInputs = ({
  tools,
  inputs,
  setInputs,
}: {
  tools: ToolDescriptor[];
  inputs: NewWorkflowInputDescriptor[];
  setInputs: (inputs: NewWorkflowInputDescriptor[]) => void;
}) => {
  const getInputSetter =
    (index: number) => (input: NewWorkflowInputDescriptor) => {
      const newInputs = inputs.slice();
      inputs[index] = input;
      setInputs(newInputs);
    };

  return (
    <div>
      {inputs.map((input, index) => (
        <WorkflowInputDescriptorEditor
          input={input}
          setInput={getInputSetter(index)}
          tools={tools}
        />
      ))}
    </div>
  );
};

export const WorkflowScreen = ({ info }: { info: WorkflowsScreenInfo }) => {
  return (
    <div>
      <AddWorkflowScreen toolList={info.tools} />
      <div>
        {info.workflows.map((workflow) => (
          <RunWorkflowScreen workflow={workflow} layers={info.layers} />
        ))}
      </div>
    </div>
  );
};
