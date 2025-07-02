import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { NumberInput, useBindedObjectState } from "./binded-input";
import { Ref } from "preact";
import { SaveButton } from "./save-button";

const GeomorphonsPolygonsScreen = ({
  innerRef,
}: {
  innerRef: Ref<HTMLInputElement>;
}) => {
  const options = useBindedObjectState({
    search: 5,
    threshold: 0.01,
    distance: 25,
    filter: 15,
  });

  const clickHandler = (name: string) =>
    client.classifyLandforms(
      name,
      options.search.value,
      options.threshold.value,
      options.distance.value,
      options.filter.value
    );

  return (
    <div>
      <NumberInput
        label="Minimum Search distance (in pixels)"
        binding={options.search}
        innerRef={innerRef}
      />
      <NumberInput
        label="Flatness threshold (in degrees)"
        binding={options.threshold}
      />
      <NumberInput
        label="Maximum search distance (in pixels)"
        binding={options.distance}
      />{" "}
      <NumberInput
        label="Window size for majority filter (in pixels)"
        binding={options.filter}
      />{" "}
      <SaveButton
        text="Run"
        prompt="Name of new polygons file"
        onSave={clickHandler}
      />
    </div>
  );
};

export const GeomorphonsPolygonsDialog = () => {
  const { open, setOpen, innerRef } = useDialog<HTMLInputElement>();
  return (
    <Dialog
      openText="Geomorphons landform classification"
      open={open}
      setOpen={setOpen}
    >
      <GeomorphonsPolygonsScreen innerRef={innerRef} />
    </Dialog>
  );
};
