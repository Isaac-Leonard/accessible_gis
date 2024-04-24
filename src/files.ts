import { open } from "@tauri-apps/api/dialog";

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
