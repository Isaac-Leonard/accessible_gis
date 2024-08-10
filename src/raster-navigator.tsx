import { Point, Classification, RasterScreenData } from "./bindings";
import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "preact/hooks";
import { client } from "./api";
import { Dialog, useDialog } from "./dialog";
import { ReprojectionDialog } from "./reprojection-dialog";
import { DemMethodsDialog } from "./dem_methods";

const tools = ["None", "Trace geometries"] as const;

export const RasterNavigator = ({ layer }: { layer: RasterScreenData }) => {
  const [tool, setTool] = useState<(typeof tools)[number]>(tools[0]);
  return (
    <div>
      <ReprojectionDialog />
      <DemMethodsDialog />
      <ClassificationDialog />
      <RasterNavigatorInner layer={layer} savePoints={() => {}} />
    </div>
  );
};

const RasterNavigatorInner = ({
  layer,
  savePoints,
}: {
  layer: RasterScreenData;
  savePoints: (points: Point[]) => void;
}) => {
  const [data, setData] = useState<number[]>([]);
  useMemo(async () => {
    let img = await client.getImagePixels();
    setData(img);
  }, []);
  let { cols, rows } = layer;
  const [showCoords, setShowCoords] = useState(true);
  const [{ x, y }, setCoords] = useState({ x: 0, y: 0 });
  const [radius, setRadius] = useState(1);
  const [points, setPoints] = useState<Point[]>([]);
  const [getCountry, setCountry] = useState(false);
  const [getTown, setTown] = useState(false);
  const [info, setInfo] = useState("");
  useEffect(() => {
    (async () => {
      if (x < cols && x >= 0 && y < rows && y >= 0) {
        if (getTown) {
          const town = await client.nearestTown({ x, y });
          setInfo(`${town?.distance ?? 0 / 1000}km from ${town?.name}`);
        } else if (getCountry) {
          const info = await client.pointInCountry({ x, y });
          setInfo(
            `In ${info?.name}, ${info?.distance ?? 0 / 1000}km from boarder`
          );
        } else {
          const val = await client.getValueAtPoint({ x, y });
          const newVal = typeof val === "number" ? val.toPrecision(4) : val;
          if (showCoords) {
            setInfo(`${newVal} at ${x}, ${rows - y}`);
          } else {
            setInfo(newVal);
          }
        }
      }
    })();
  }, [cols, rows, radius, x, y, showCoords]);
  return (
    <div
      onKeyDown={(e) => {
        if (e.key === "M") {
          e.preventDefault();
          client.getPointOfMaxValue().then((p) => {
            if (p !== null) {
              setCoords(p);
            }
          });
        }
        if (e.key === "m") {
          e.preventDefault();
          client.getPointOfMinValue().then((p) => {
            if (p !== null) {
              setCoords(p);
            }
          });
        }
        if (e.key === "c") {
          e.preventDefault();
          setCountry(true);
          setTown(false);
        }
        if (e.key === "t") {
          e.preventDefault();
          setTown(true);
          setCountry(false);
        }
        if (e.key === "p") {
          e.preventDefault();
          setPoints([...points, { x, y }]);
        }
        if (e.key === "s" && e.ctrlKey) {
          savePoints(points);
        }
      }}
    >
      <input
        type="number"
        value={radius}
        onChange={(e) => setRadius(Number(e.currentTarget.value))}
      />
      <label>
        Show coords?{" "}
        <input
          type="checkbox"
          defaultChecked={true}
          onChange={(e) => setShowCoords(e.currentTarget.checked)}
        />
      </label>
      <button
        aria-checked={layer.display}
        role="switch"
        onClick={() => client.setDisplay()}
      >
        Display
      </button>
      <div onKeyDown={coordinateArrowHandler(x, y, radius, setCoords)}>
        <CoordinateButtons x={x} y={y} radius={radius} setCoords={setCoords} />

        <p role="status">{info}</p>
      </div>
    </div>
  );
};

type CoordProps = {
  x: number;
  y: number;
  radius: number;
  setCoords: (_: { x: number; y: number }) => void;
};

function CoordinateButtons({ x, y, radius, setCoords }: CoordProps) {
  return (
    <div>
      <button onClick={() => setCoords({ x, y: y - radius })}>Up</button>
      <button onClick={() => setCoords({ x, y: y + radius })}>Down</button>
      <button onClick={() => setCoords({ x: x - radius, y })}>Left</button>
      <button onClick={() => setCoords({ x: x + radius, y })}>Right</button>
    </div>
  );
}

function coordinateArrowHandler(
  x: number,
  y: number,
  radius: number,
  setCoords: (_: { x: number; y: number }) => void
) {
  return (e: KeyboardEvent) => {
    if (e.key.startsWith("Arrow")) {
      e.preventDefault();
      switch (e.key) {
        case "ArrowUp":
          setCoords({ x, y: y + radius });
          break;
        case "ArrowDown":
          setCoords({ x, y: y - radius });
          break;
        case "ArrowLeft":
          setCoords({ x: x - radius, y });
          break;
        case "ArrowRight":
          setCoords({ x: x + radius, y });
          break;
      }
    }
  };
}

const SaveScreen = ({
  points,
  done,
  cancel,
}: {
  points: Point[];
  done: () => void;
  cancel: () => void;
}) => {
  return (
    <div>
      <button onClick={cancel}>Cancel</button>
      <button onClick={done}>Done</button>
    </div>
  );
};

const ClassificationDialog = () => {
  const { open, innerRef, setOpen } = useDialog();

  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Classify raster"
    >
      <ClassificationScreen onClassify={() => setOpen(false)} />
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
      <button
        onClick={async () => {
          const file = await save({ title: "Classified raster destination" });
          if (file !== null) {
            await client.classifyCurrentRaster(
              file,
              classifications.map(mapClassification)
            );
            onClassify();
          }
        }}
      >
        Save classified raster
      </button>
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
