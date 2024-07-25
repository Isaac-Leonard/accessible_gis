import { save } from "@tauri-apps/plugin-dialog";

export const SaveButton = ({ onSave }: { onSave: (name: string) => void }) => {
  const clickHandler = async () => {
    const name = await save();
    if (name !== null) {
      onSave(name);
    }
  };

  return <button onClick={clickHandler}>Save</button>;
};
