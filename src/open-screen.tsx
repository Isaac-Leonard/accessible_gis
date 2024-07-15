import { useState } from "preact/hooks";
import { Drawer, useDrawer } from "./drawer";
import { load, loadFile, openFile } from "./files";
import { client } from "./api";

export const OpenDatasetDialog = () => {
  const { open, setOpen, innerRef } = useDrawer();

  const fileHandler = () => {
    load();
    setOpen(false);
  };

  return (
    <Drawer open={open} setOpen={setOpen} openText="Open dataset">
      <button ref={innerRef} onClick={fileHandler}>
        Open file
      </button>
      <OpenLink />
    </Drawer>
  );
};

const OpenLink = () => {
  const { open, setOpen, innerRef } = useDrawer();
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
          loadFile(url);
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
