import { useSignal } from "@preact/signals";
import { SaveButton } from "./save-button";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";

export const LayerSimplifier = () => {
  const tolerance = useSignal<string>("");

  const handleSave = (name: string) => {
    const val = Number.parseFloat(tolerance.value);
    if (val !== 0) {
      client.simplifyLayer(val, name);
    }
  };
  return (
    <div>
      <label>
        <input
          type="number"
          value={tolerance}
          onInput={(e) => (tolerance.value = e.currentTarget.value)}
        />
      </label>
      <SaveButton onSave={handleSave} />
    </div>
  );
};

export const LayerSimplifierDialog = () => {
  const { open, setOpen, innerRef } = useDialog();
  return (
    <Dialog
      modal={true}
      openText="Simplify geometries"
      open={open}
      setOpen={setOpen}
    >
      <LayerSimplifier />
    </Dialog>
  );
};