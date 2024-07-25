import { useState } from "preact/hooks";
import { VectorScreenData } from "./bindings";
import { SaveButton } from "./save-button";

export const FeatureCopyier = ({ layer }: { layer: VectorScreenData }) => {
  const features = layer.feature_names as string[];
  const [selectedFeatures, setSelectedFeatures] = useState<number[]>([]);
  return (
    <div>
      <MultiFeaturePicker
        features={features}
        selectedFeatures={selectedFeatures}
        setSelectedFeatures={setSelectedFeatures}
      />
      <SaveButton />
    </div>
  );
};

// Note that selected features must be numbers as there can be many features with the same name and so we must keep track of their indexes

type MultiFeaturePickerProps = {
  features: string[];
  selectedFeatures: number[];
  setSelectedFeatures: (features: number[]) => void;
};

const MultiFeaturePicker = ({
  features,
  selectedFeatures,
  setSelectedFeatures,
}: MultiFeaturePickerProps) => {
  return (
    <div>
      {features.map((feature, index) => {
        const selectedIndex = selectedFeatures.indexOf(index);
        return (
          <input
            type="checkbox"
            value={feature}
            checked={selectedIndex !== -1}
            onInput={() => {
              if (selectedIndex === -1) {
                setSelectedFeatures([...selectedFeatures, index]);
              } else {
                const newSelectedFeatures = [...selectedFeatures];
                newSelectedFeatures.splice(selectedIndex, 1);
                setSelectedFeatures(newSelectedFeatures);
              }
            }}
          />
        );
      })}
    </div>
  );
};
