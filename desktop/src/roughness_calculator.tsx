import { client } from "./api";
import { SaveButton } from "./save-button";

export const RoughnessCalculator = () => {
  return (
    <SaveButton
      text="Calculate the roughness"
      prompt="Name of new roughness file"
      onSave={client.calcRoughness}
    />
  );
};
