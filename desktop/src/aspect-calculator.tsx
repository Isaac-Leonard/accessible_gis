import { client } from "./api";
import { SaveButton } from "./save-button";

export const AspectCalculator = () => {
  return (
    <SaveButton
      text="Calculate the aspect"
      prompt="Name of new aspect file"
      onSave={client.calcAspect}
    />
  );
};
