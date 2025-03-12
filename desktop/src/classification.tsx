import { useState } from "preact/hooks";
import { client } from "./api";
import { Classification } from "./bindings";
import { SaveButton } from "./save-button";
import { Dialog, useDialog } from "./dialog";
import { H, Section } from "react-headings";

export const ClassificationDialog = () => {
  const { open, innerRef, setOpen } = useDialog<HTMLHeadingElement>();

  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Classify raster"
    >
      <Section component={<H ref={innerRef}>Classify Cells in Raster</H>}>
        <ClassificationScreen onClassify={() => setOpen(false)} />
      </Section>
    </Dialog>
  );
};

type StringClassification = { target: string; min: string; max: string };

const ClassificationScreen = ({ onClassify }: { onClassify: () => void }) => {
  const [classifications, setClassifications] = useState<
    StringClassification[]
  >([]);
  console.log(classifications);
  const handleInput =
    (index: number, key: keyof Classification) => (value: string) => {
      const newClassifications = [...classifications];
      newClassifications[index] = {
        ...newClassifications[index],
        [key]: Number(value),
      };
      setClassifications(newClassifications);
    };

  return (
    <div>
      <table>
        <thead>
          <tr>
            <th>Class</th> <th>Target</th>
            <th>Min</th>
            <th>Max</th>
          </tr>
        </thead>
        <tbody>
          {" "}
          {classifications.map(({ target, min, max }, i) => (
            <tr>
              <td>{i}</td>
              <td>
                <ClassificationInput
                  value={target}
                  setValue={handleInput(i, "target")}
                />
              </td>
              <td>
                <ClassificationInput
                  value={min}
                  setValue={handleInput(i, "min")}
                />
              </td>
              <td>
                <ClassificationInput
                  value={max}
                  setValue={handleInput(i, "max")}
                />
              </td>
            </tr>
          ))}
        </tbody>{" "}
      </table>
      <button
        onClick={() =>
          setClassifications([
            ...classifications,
            { target: "", min: "", max: "" },
          ])
        }
      >
        Add Classification
      </button>
      <SaveButton
        text="Save classified raster"
        prompt="Classified raster destination"
        onSave={async (file) => {
          await client.classifyCurrentRaster(
            file,
            classifications.map(mapClassification)
          );
          onClassify();
        }}
      />
    </div>
  );
};

type ClassificationInputProps = {
  value: string;
  setValue: (value: string) => void;
};

const ClassificationInput = ({ value, setValue }: ClassificationInputProps) => {
  return (
    <input value={value} onInput={(e) => setValue(e.currentTarget.value)} />
  );
};

const mapClassification = (
  classification: StringClassification
): Classification => ({
  target: Number(classification.target),
  min: Number(classification.min),
  max: Number(classification.max),
});
