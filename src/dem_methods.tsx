import { AspectCalculator } from "./aspect-calculator";
import { Dialog, useDialog } from "./dialog";
import { RoughnessCalculator } from "./roughness_calculator";
import { SlopeCalculator } from "./slope-calculator";

export const DemMethodsDialog = () => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Dem operations"
    >
      <SlopeCalculator />
      <AspectCalculator />
      <RoughnessCalculator />
      <button onClick={() => setOpen(false)}>Close</button>
    </Dialog>
  );
};
