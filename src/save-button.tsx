import { DialogFilter, save } from "@tauri-apps/plugin-dialog";

type SaveButtonProps = {
  text?: string;
  onSave: (name: string) => void;
  prompt?: string;
  filters?: DialogFilter[];
};

export const SaveButton = ({
  onSave,
  text,
  prompt,
  filters,
}: SaveButtonProps) => {
  if (typeof text !== "string") {
    text = "Save";
  }
  const clickHandler = async () => {
    const name = await save({ title: prompt, filters });
    if (name !== null) {
      onSave(name);
    }
  };

  return <button onClick={clickHandler}>{text}</button>;
};
