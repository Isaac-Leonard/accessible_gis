import { useState } from "preact/hooks";
import {
  ToolDescriptor,
  NewWorkflowInputDescriptor,
  WorkflowsScreenInfo,
  NewToolCall,
  ToolInputTypeDiscriminants,
  ToolInputType,
  ToolPresetParameterValueDiscriminants,
  WorkflowInputRuntimeValueDescriptor,
} from "./bindings";
import { IndexedOptionPicker } from "./option-picker";
import { Dialog, useDialog } from "./dialog";
import { client } from "./api";
import { Checkbox, Input as TextInput } from "./binded-input";
import {
  presetInputFromDiscriminant,
  PresetInputTypeSelector,
  PresetInputValueEditor,
  ToolDialog,
  ToolInputTypeDiscriminantSelector,
  toolInputTypeFromDiscriminant,
} from "./tools-screen";

const AddWorkflowScreen = ({ toolList }: { toolList: ToolDescriptor[] }) => {
  const [label, setLabel] = useState("");
  const [inputs, setInputs] = useState<NewWorkflowInputDescriptor[]>([]);
  console.log(inputs);
  const [tools, setTools] = useState<NewToolCall[]>([]);
  const { open, setOpen } = useDialog();
  return (
    <Dialog open={open} setOpen={setOpen} modal={true} openText="Add workflow">
      <h3>Workflow creation dialog</h3>
      <TextInput
        label="Display name for workflow"
        binding={{ value: label, setValue: setLabel }}
      />
      <ListOfInputs inputs={inputs} setInputs={setInputs} />
      <button
        onClick={() =>
          setInputs([
            ...inputs,
            {
              label: "",
              value: {
                type: "Runtime",
                value: { param_type: { type: "Int" }, optional: false },
              },
            },
          ])
        }
      >
        Add input
      </button>
      <ul>
        {tools.map((toolCall, toolCallIndex) => {
          const tool = toolList.find((tool) => tool.id === toolCall.tool)!;
          return (
            <li key={toolCall.tool + toolCallIndex}>
              {tool.label}:
              {tool.inputs.map((toolInput, toolInputIndex) => (
                <div>
                  <IndexedOptionPicker
                    prompt={toolInput.label}
                    options={inputs.map((input) => input.label)}
                    index={toolCall.inputs[toolInputIndex].input}
                    setIndex={(inputIndex) => {
                      const connection = {
                        parameter: toolCall.inputs[toolInputIndex].parameter,
                        input: inputIndex,
                      };
                      const connections = toolCall.inputs.slice();
                      connections[toolInputIndex] = connection;
                      const toolCallReplacement = {
                        tool: toolCall.tool,
                        inputs: connections,
                      };
                      const toolCallReplacements = tools.slice();
                      toolCallReplacements[toolCallIndex] = toolCallReplacement;
                      setTools(toolCallReplacements);
                    }}
                    emptyText="No workflow inputs added yet, maybe add one?"
                  />
                </div>
              ))}
            </li>
          );
        })}
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
        Create workflow
      </button>
    </Dialog>
  );
};

const AddTool = ({
  toolList,
  addTool,
}: {
  toolList: ToolDescriptor[];
  addTool: (tool: NewToolCall) => void;
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
            addTool({
              tool: toolList[tool].id,
              inputs: toolList[tool].inputs.map((input) => ({
                parameter: input.id,
                input: 0,
              })),
            });
          }
        }}
      >
        Add
      </button>
    </Dialog>
  );
};

type WorkflowInputTypePickerProps = {
  type: ToolInputType;
  setType: (type: ToolInputType) => void;
};

const WorkflowInputTypePicker = ({
  type,
  setType,
}: WorkflowInputTypePickerProps) => {
  return (
    <div>
      <ToolInputTypeDiscriminantSelector
        prompt="Input type"
        binding={{
          value: type.type,
          setValue: (discriminant: ToolInputTypeDiscriminants) =>
            setType(toolInputTypeFromDiscriminant(discriminant)),
        }}
      />
    </div>
  );
};

const WorkflowInputDescriptorEditor = ({
  input,
  setInput,
}: {
  input: NewWorkflowInputDescriptor;
  setInput: (input: NewWorkflowInputDescriptor) => void;
}) => {
  const setType = (value: NewWorkflowInputDescriptor["value"]) =>
    setInput({ label: input.label, value });

  const setLabel = (label: string) => setInput({ label, value: input.value });

  return (
    <div>
      <TextInput
        label="Label"
        binding={{ value: input.label, setValue: setLabel }}
      />
      <button
        role="switch"
        aria-checked={input.value.type === "Preset"}
        onClick={() =>
          setType(
            input.value.type === "Preset"
              ? {
                  type: "Runtime",
                  value: { optional: false, param_type: { type: "String" } },
                }
              : { type: "Preset", value: { type: "String", value: "" } }
          )
        }
      >
        Preset value
      </button>
      {input.value.type === "Runtime" ? (
        <div>
          <Checkbox
            label="Optional"
            binding={{
              value: input.value.value.optional,
              setValue: (value) =>
                setType({
                  type: "Runtime",
                  value: {
                    ...(input.value
                      .value as WorkflowInputRuntimeValueDescriptor),
                    optional: value,
                  },
                }),
            }}
          />
          <WorkflowInputTypePicker
            type={input.value.value.param_type}
            setType={(type) =>
              setType({
                type: "Runtime",
                value: {
                  param_type: type,
                  optional: (
                    input.value.value as WorkflowInputRuntimeValueDescriptor
                  ).optional,
                },
              })
            }
          />
        </div>
      ) : (
        <>
          <PresetInputTypeSelector
            prompt="Type of preset input"
            binding={{
              value: input.value.value.type,
              setValue: (discriminant: ToolPresetParameterValueDiscriminants) =>
                setType({
                  type: "Preset",
                  value: presetInputFromDiscriminant(discriminant),
                }),
            }}
          />
          <PresetInputValueEditor
            input={input.value.value}
            setInput={(value: typeof input.value.value) =>
              setType({ type: "Preset", value })
            }
          />
        </>
      )}
    </div>
  );
};

const ListOfInputs = ({
  inputs,
  setInputs,
}: {
  inputs: NewWorkflowInputDescriptor[];
  setInputs: (inputs: NewWorkflowInputDescriptor[]) => void;
}) => {
  const getInputSetter =
    (index: number) => (input: NewWorkflowInputDescriptor) => {
      console.log("set input called for input " + index);
      console.log(input);
      const newInputs = inputs.slice();
      newInputs[index] = input;
      setInputs(newInputs);
    };

  return (
    <div>
      {inputs.map((input, index) => (
        <WorkflowInputDescriptorEditor
          input={input}
          setInput={getInputSetter(index)}
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
          <ToolDialog
            tool={workflow}
            layers={info.layers}
            run={client.runWorkflow}
          />
        ))}
      </div>
    </div>
  );
};
