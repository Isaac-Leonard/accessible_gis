import { useState } from "preact/hooks";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { NumberInput, useBindedObjectProperties } from "./binded-input";
import { Ref } from "preact";
import { ColourPicker } from "./vector/settings";
import { TouchDeviceState } from "./bindings";

export const TouchDeviceScreen = ({ state }: { state: TouchDeviceState }) => {
  const { open, setOpen, innerRef } = useDialog<HTMLInputElement>();
  return (
    <div>
      <Dialog
        open={open}
        setOpen={setOpen}
        openText="Focus screen on coordinates"
        modal={true}
      >
        <FocusBoxScreen onClose={() => setOpen(false)} innerRef={innerRef} />
      </Dialog>
      <h3>Settings</h3>
      <ColourPicker
        prompt="Background colour"
        colour={state.settings.background_colour}
        onDone={client.setBackgroundColour}
      />
    </div>
  );
};

type FocusBoxScreenProps = {
  onClose: () => void;
  innerRef?: Ref<HTMLInputElement>;
};

const FocusBoxScreen = ({ onClose, innerRef }: FocusBoxScreenProps) => {
  const [bounds, setBounds] = useState({
    minLon: 0,
    maxLon: 0,
    minLat: 0,
    maxLat: 0,
  });
  const x = useBindedObjectProperties(bounds, setBounds);

  const clickHandler = () => {
    client.focusBox([
      bounds.minLon,
      bounds.minLat,
      bounds.maxLon,
      bounds.maxLat,
    ]);
    onClose();
  };

  return (
    <div>
      <NumberInput
        binding={x.minLon}
        label="Minimum longitude"
        innerRef={innerRef}
      />
      <NumberInput binding={x.maxLon} label="Maximum longitude" />{" "}
      <NumberInput binding={x.minLat} label="Minimum Latitude" />
      <NumberInput binding={x.maxLat} label="Maximum latitude" />{" "}
      <button onClick={clickHandler}>Focus bounds</button>
    </div>
  );
};
