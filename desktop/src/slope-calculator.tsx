import { client } from "./api";
import { SaveButton } from "./save-button";

export const SlopeCalculator = () => {
  return (
    <SaveButton
      text="Calculate the slope"
      prompt="Name of new slope file"
      onSave={client.calcSlope}
    />
  );
};
