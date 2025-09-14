import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { GeomorphonsPolygonsDialog } from "./geomorphons-polygons";
import { SaveButton } from "./save-button";

export const DemMethodsDialog = () => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Dem operations"
    >
      <SaveButton
        text="Calculate the slope"
        prompt="Name of new slope file"
        onSave={client.calcSlope}
      />
      <SaveButton
        text="Calculate the aspect"
        prompt="Name of new aspect file"
        onSave={client.calcAspect}
      />
      <SaveButton
        text="Calculate the roughness"
        prompt="Name of new roughness file"
        onSave={client.calcRoughness}
      />
      <SaveButton
        text="Calculate the Topographic Position Index"
        prompt="Name of new TPI file"
        onSave={client.calcTpi}
      />
      <SaveButton
        text="Calculate the Terrain Ruggedness Index"
        prompt="Name of new TRI file"
        onSave={client.calcTri}
      />
      <SaveButton
        text="Generate hillshade"
        prompt="Name of new hillshade file"
        onSave={client.calcHillshade}
      />
      <SaveButton
        text="Generate colour relief"
        prompt="Name of new colour relief file"
        onSave={client.calcColorRelief}
      />
      <GeomorphonsPolygonsDialog />
      <button onClick={() => setOpen(false)}>Close</button>
    </Dialog>
  );
};
