import { client } from "./api";
import { VectorScreenData } from "./bindings";
import { IndexedOptionPicker } from "./option-picker";

type FeaturePickerProps = {
  layer: VectorScreenData;
};

export const FeaturePicker = ({ layer }: FeaturePickerProps) => {
  const { feature_names, feature_idx } = layer;
  return (
    <IndexedOptionPicker
      options={feature_names?.filter((x): x is string => x !== null) ?? []}
      index={feature_idx}
      setIndex={client.setFeatureIndex}
      prompt="Select a feature to examine"
      emptyText="This layer has no features"
    />
  );
};
