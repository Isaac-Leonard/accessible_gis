import {
  AudioSettings,
  Point,
  RasterScreenData,
  RasterScreenMetadata,
} from "./bindings";
import { useEffect, useState } from "preact/hooks";
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
      <RasterMetadataDialog metadata={layer.metadata} />
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
      <AudioSettingsDialog settings={layer.audio_settings} />
      <RasterNavigatorInner layer={layer} />
    </div>
  );
};

const RasterNavigatorInner = ({ layer }: { layer: RasterScreenData }) => {
  return (
    <div>
      <button
        aria-checked={layer.display}
        role="switch"
        onClick={() =>
          client.setDisplayRaster(
            layer.display
              ? null
              : { dataset: layer.dataset_index, band: layer.layer_index }
          )
        }
      >
        Display
      </button>
      {layer.display ? (
        <>
          <button onClick={() => client.focusDataset()}>Zoom to raster</button>{" "}
          <RenderMethodsSelector
            prompt="Load data on touch device as"
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
        </>
      ) : (
        ""
      )}
      <PixelExplorer layer={layer} />
    </div>
  );
};

const PixelExplorer = ({ layer }: { layer: RasterScreenData }) => {
  const [showCoords, setShowCoords] = useState(true);
  const [points, setPoints] = useState<Point[]>([]);
  const [{ x, y }, setCoords] = useState({ x: 0, y: 0 });
  const [radius, setRadius] = useState(1);
  const { open, setOpen } = useDialog();
  let { cols, rows } = layer.metadata;
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

  const keyHandler = (e: KeyboardEvent) => {
    coordinateArrowHandler(x, y, radius, setCoords)(e);
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
    // TODO: Implement a way to properly build up geometries manually by examining raster data.
    if (e.key === "s" && e.ctrlKey) {
      // savePoints(points);
    }
  };

  return (
    <Dialog
      modal={true}
      openText="Explore pixels"
      open={open}
      setOpen={setOpen}
    >
      <div onKeyDown={keyHandler}>
        <input
          type="number"
          value={radius}
          onChange={(e) => setRadius(Number(e.currentTarget.value))}
          autofocus={true}
        />
        <label>
          Show coords?{" "}
          <input
            type="checkbox"
            defaultChecked={true}
            onChange={(e) => setShowCoords(e.currentTarget.checked)}
          />
        </label>
        <CoordinateButtons x={x} y={y} radius={radius} setCoords={setCoords} />

        <p role="status">{info}</p>
      </div>
    </Dialog>
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

const RasterMetadataDialog = ({
  metadata,
}: {
  metadata: RasterScreenMetadata;
}) => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog openText="Band Metadata" modal={true} open={open} setOpen={setOpen}>
      <h3>Size and projection</h3>
      <div>
        Size: {metadata.cols} by {metadata.rows}
      </div>
      <div>srs: {metadata.srs}</div>
      {Object.entries(metadata.other).map(([domain, items]) => (
        <div>
          <h3>{domain === "" ? "Route metadata" : domain}</h3>
          {Object.entries(items).map(([name, value]) => (
            <div>
              {name} = {value}
            </div>
          ))}
        </div>
      ))}
    </Dialog>
  );
};
