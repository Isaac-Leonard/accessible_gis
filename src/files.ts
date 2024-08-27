import { open } from "@tauri-apps/plugin-dialog";
import { client } from "./api";

export const openFile = async (prompt?: string): Promise<null | string> => {
  const selected = await open({
    title: prompt,
    multiple: false,
    filters: [],
    directory: true,
  });
  if (Array.isArray(selected)) {
    return selected[0];
  } else if (selected === null) {
    return null;
  } else {
    return selected;
  }
};

export const loadFile = async (name: string): Promise<string | null> => {
  return await client.loadFile(name).catch((e) => e as unknown as string);
};

export async function load() {
  const file = await openFile();
  if (file !== null) {
    await client.loadFile(file);
  }
}
