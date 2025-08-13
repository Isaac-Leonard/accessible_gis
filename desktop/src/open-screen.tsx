import { useState } from "preact/hooks";
import { Drawer, useDrawer } from "./drawer";
import { load, loadMulti } from "./files";
import { client } from "./api";

export const OpenDatasetDialog = () => {
  const { open, setOpen, innerRef } = useDrawer<HTMLButtonElement>();

  const datasetHandler = () => {
    load();
    setOpen(false);
  };

  const multiDatasetHandler = () => {
    loadMulti();
    setOpen(false);
  };

  return (
    <Drawer open={open} setOpen={setOpen} openText="Open dataset">
      <button ref={innerRef} onClick={datasetHandler}>
        Open dataset
      </button>
      <button ref={innerRef} onClick={multiDatasetHandler}>
        Open multi dataset
      </button>
      <OpenLink />
    </Drawer>
  );
};

const OpenLink = () => {
  const { open, setOpen, innerRef } = useDrawer<HTMLInputElement>();
  const [url, setUrl] = useState("");
  return (
    <Drawer
      open={open}
      setOpen={setOpen}
      openText="Connect to dataset with url"
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          client.loadFile(url);
          setOpen(false);
        }}
      >
        <label>
          Url to connect to:
          <input
            ref={innerRef}
            value={url}
            onInput={(e) => setUrl(e.currentTarget.value)}
          />
        </label>
        <input type="submit" />
      </form>
    </Drawer>
  );
};
