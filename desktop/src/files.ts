import { DialogFilter, open, save } from "@tauri-apps/plugin-dialog";
import { client } from "./api";

export const openFile = async (
  prompt?: string,
  filters?: DialogFilter[]
): Promise<null | string> => {
  const selected = await open({
    title: prompt,
    multiple: false,
    filters: filters,
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

export async function load() {
  const file = await openFile();
  if (file !== null) {
    await client.loadFile(file);
  }
}

export async function loadMulti() {
  const file = await openFile();
  if (file !== null) {
    await client.loadDatasetMulti(file);
  }
}

export const newProject = () =>
  save({
    title: "Project location",
    defaultPath: "accessible_gis_project.json",
  }).then((name) => {
    if (name !== null) {
      return client.createProject(name);
    }
  });
