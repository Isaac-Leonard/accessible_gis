import { DialogFilter, save } from "@tauri-apps/plugin-dialog";
import { openFile } from "./files";

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

type LoadButtonProps = {
  text?: string;
  onLoad: (name: string) => void;
  prompt?: string;
  filters?: DialogFilter[];
};

export const LoadButton = ({
  onLoad,
  text,
  prompt,
  filters,
}: LoadButtonProps) => {
  if (typeof text !== "string") {
    text = "Load";
  }
  const clickHandler = async () => {
    const name = await openFile(prompt, filters);
    if (name !== null) {
      onLoad(name);
    }
  };

  return <button onClick={clickHandler}>{text}</button>;
};
