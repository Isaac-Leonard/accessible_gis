import { useState } from "preact/hooks";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { SaveButton } from "./save-button";
import { SrsSelector, defaultSrs } from "./srs-selector";

export const ReprojectionDialog = () => {
  const { open, setOpen, innerRef } = useDialog<HTMLDivElement>();
  const [srs, setSrs] = useState(defaultSrs());
  return (
    <Dialog
      modal={true}
      openText="Reproject dataset"
      open={open}
      setOpen={setOpen}
    >
      <div ref={innerRef}>Target srs:</div>
      <SrsSelector srs={srs} setSrs={setSrs} />
      <SaveButton onSave={(name) => client.reprojectLayer(srs, name)} />
    </Dialog>
  );
};
