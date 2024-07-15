import { OptionPicker } from "./option-picker";

const Tools = ["TraceGeometries"] as const;

export const ToolScreen = () => {
  return (
    <div>
      <ToolList />
    </div>
  );
};

export const ToolList = () => {
  return (
    <OptionPicker
      options={Tools}
      emptyText="This should not be empty, no tools found"
    />
  );
};

export const ActiveToolList = () => {
  return <OptionPicker emptyText="This should not be empty, no tools found" />;
};
