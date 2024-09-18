import { useState } from "preact/hooks";
import { FeatureIdentifier, VectorScreenData } from "./bindings";
import { SaveButton } from "./save-button";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { H, Section } from "react-headings";

export const FeatureCoppier = ({ layer }: { layer: VectorScreenData }) => {
  const features = layer.features;
  const [selectedFeatures, setSelectedFeatures] = useState<number[]>([]);
  return (
    <div>
      <MultiFeaturePicker
        features={features}
        selectedFeatures={selectedFeatures}
        setSelectedFeatures={setSelectedFeatures}
      />
      <SaveButton
        onSave={(name) => client.copyFeatures(selectedFeatures, name)}
      />
    </div>
  );
};

type MultiFeaturePickerProps = {
  features: FeatureIdentifier[];
  selectedFeatures: number[];
  setSelectedFeatures: (features: number[]) => void;
};

const MultiFeaturePicker = ({
  features,
  selectedFeatures,
  setSelectedFeatures,
}: MultiFeaturePickerProps) => {
  const handleChange = (fid: number) => {
    if (selectedFeatures.includes(fid)) {
      setSelectedFeatures(
        selectedFeatures.filter((feature) => feature !== fid)
      );
    } else {
      setSelectedFeatures([...selectedFeatures, fid]);
    }
  };

  return (
    <div>
      {features.map(({ name, fid }) => {
        return (
          <label>
            {name}
            <input
              key={fid}
              type="checkbox"
              checked={selectedFeatures.includes(fid)}
              onChange={() => handleChange(fid)}
            />
          </label>
        );
      })}
    </div>
  );
};

export const FeatureCoppierDialog = ({
  layer,
}: {
  layer: VectorScreenData;
}) => {
  const { open, setOpen, innerRef } = useDialog<HTMLHeadingElement>();
  return (
    <Dialog
      modal={true}
      openText="Copy features to new dataset"
      open={open}
      setOpen={setOpen}
    >
      <Section component={<H ref={innerRef}>Copy Features</H>}>
        <FeatureCoppier layer={layer} />
      </Section>
    </Dialog>
  );
};
