import { Srs } from "./bindings";
import { OptionPicker } from "./option-picker";

const srsKind = ["Epsg", "Esri", "Proj", "Wkt"] as const;

type SrsSelectorProps = { srs: Srs; setSrs: (srs: Srs) => void };

export const SrsSelector = ({ srs, setSrs }: SrsSelectorProps) => {
  return (
    <div>
      <OptionPicker
        prompt="SRS type:"
        options={srsKind}
        selectedOption={srs.type}
        setOption={(type) => setSrs(defaultSrsFor(type))}
        emptyText="Something has gone very wrong"
      />
      <SrsValueInput srs={srs} setSrs={setSrs} />
    </div>
  );
};

const SrsValueInput = ({ srs, setSrs }: SrsSelectorProps) => {
  switch (srs.type) {
    case "Wkt":
    case "Proj":
    case "Esri":
      return (
        <label>
          <input
            value={srs.value}
            onInput={(e) =>
              setSrs({ type: srs.type, value: e.currentTarget.value })
            }
          />
        </label>
      );
    case "Epsg":
      return (
        <label>
          <input
            value={srs.value}
            onInput={(e) =>
              setSrs({ type: srs.type, value: Number(e.currentTarget.value) })
            }
          />
        </label>
      );
  }
};

const defaultSrsFor = (kind: Srs["type"]): Srs => {
  switch (kind) {
    case "Esri":
    case "Proj":
    case "Wkt":
      return { type: kind, value: "" };
    case "Epsg":
      return { type: kind, value: 4326 };
  }
};

export const defaultSrs = (): Srs => defaultSrsFor("Epsg");
