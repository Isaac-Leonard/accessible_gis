import { AudioSettings, Point, RasterScreenData } from "./bindings";
import { useEffect, useMemo, useState } from "preact/hooks";
import { client } from "./api";
import { ReprojectionDialog } from "./reprojection-dialog";
import { DemMethodsDialog } from "./dem_methods";
import { ClassificationDialog } from "./classification";
import { SaveButton } from "./save-button";
import { RenderMethodsSelector } from "./render_methods_selector";
import { Dialog, useDialog } from "./dialog";
import { AudioSettingsScreen } from "./settings-screen";

export const RasterNavigator = ({ layer }: { layer: RasterScreenData }) => {
  return (
    <div>
      <ReprojectionDialog />
      <DemMethodsDialog />
      <ClassificationDialog />
      <SaveButton
        text="Generate pixel counts report"
        prompt="Counts report file"
        filters={[{ name: "csv", extensions: ["csv"] }]}
        onSave={client.generateCountsReport}
      />
      <button onClick={() => client.playAsSound()}>Play audio</button>
      <button onClick={() => client.playHistogram()}>
        Play audio Histogram
      </button>
      <RenderMethodsSelector
        binding={{
          value: layer.render_method,
          setValue: client.setCurrentRenderMethod,
        }}
      />
      <label>
        Enable OCR when displayed?
        <input
          role="switch"
          type="checkbox"
          checked={layer.ocr}
          aria-pressed={layer.ocr}
          onChange={(e) => client.setCurrentOcr(e.currentTarget.checked)}
        />
      </label>
      <AudioSettingsDialog settings={layer.audio_settings} />
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
            setInfo(`${newVal ?? "No data"} at ${x}, ${rows - y}`);
          } else {
            setInfo(newVal ?? "No data");
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

const AudioSettingsDialog = ({ settings }: { settings: AudioSettings }) => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog openText="Customise Audio Settings" open={open} setOpen={setOpen}>
      <AudioSettingsScreen
        settings={settings}
        setSettings={client.setCurrentAudioSettings}
      />
    </Dialog>
  );
};
