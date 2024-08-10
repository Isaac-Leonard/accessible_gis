import { save } from "@tauri-apps/plugin-dialog";

type SaveButtonProps = {
  text?: string;
  onSave: (name: string) => void;
  prompt?: string;
};

export const SaveButton = ({ onSave, text, prompt }: SaveButtonProps) => {
  if (typeof text !== "string") {
    text = "Save";
  }
  const clickHandler = async () => {
    const name = await save({ title: prompt });
    if (name !== null) {
      onSave(name);
    }
  };

  return <button onClick={clickHandler}>{text}</button>;
};
