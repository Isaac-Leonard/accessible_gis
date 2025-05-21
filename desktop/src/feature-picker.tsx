import { client } from "./api";
import { VectorScreenData } from "./bindings";
import { IndexedOptionPicker } from "./option-picker";

type FeaturePickerProps = {
  layer: VectorScreenData;
};

export const FeaturePicker = ({ layer }: FeaturePickerProps) => {
  const fid = layer.feature?.fid ?? null;
  return (
    <IndexedOptionPicker
      options={
        layer.features
          .map((x) => x.name)
          .filter((x): x is string => x !== null) ?? []
      }
      index={layer.features.findIndex((x) => x.fid === fid)}
      setIndex={(idx) => client.setFeatureIndex(layer.features[idx].fid)}
      prompt="Select a feature to examine"
      emptyText="This layer has no features"
    />
  );
};
