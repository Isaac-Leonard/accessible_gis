import * as Path from "@tauri-apps/api/path";
import {
  AudioSettings,
  Point,
  RasterScreenData,
  RasterScreenMetadata,
  DatasetMetadata,
  AudioTable,
  ColourTable,
  AudioTypeDiscriminants,
  AudioType,
} from "./bindings";
import { useEffect, useState } from "preact/hooks";
import { client } from "./api";
import { ReprojectionDialog } from "./reprojection-dialog";
import { DemMethodsDialog } from "./dem_methods";
import { ClassificationDialog } from "./classification";
import { LoadButton, SaveButton } from "./save-button";
import { RenderMethodsSelector } from "./render_methods_selector";
import { Dialog, useDialog } from "./dialog";
import { AudioSettingsScreen } from "./settings-screen";
import {
  bindedSelectorFactory,
  IndexedOptionPicker,
  OptionPicker,
} from "./option-picker";
import { NumberInput, Input as TextInput } from "./binded-input";
import { convertFileSrc } from "@tauri-apps/api/core";

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
      <AudioTableManager
        audioTable={layer.audio_table}
        colourTable={layer.colour_table}
      />
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
      <GdalMetadataViewer metadata={metadata.other} />
    </Dialog>
  );
};

export const GdalMetadataViewer = ({
  metadata,
}: {
  metadata: DatasetMetadata;
}) => (
  <div>
    {Object.entries(metadata).map(([domain, items]) => (
      <div>
        <h3>{domain === "" ? "Route metadata" : domain}</h3>
        {Object.entries(items).map(([name, value]) => (
          <div>
            {name} = {value}
          </div>
        ))}
      </div>
    ))}
  </div>
);

type AudioTableManagerProps = {
  audioTable: AudioTable | null;
  colourTable: ColourTable | null;
};

const AudioTableManager = ({
  audioTable,
  colourTable,
}: AudioTableManagerProps) => {
  const { open, setOpen } = useDialog();
  const { open: openEditor, setOpen: setOpenEditor } = useDialog();
  const createAudioTable = () => {
    client.setAudioTable({
      entries: colourTable
        ? colourTable.entries.map(() =>
            getDefaultAudioTypeFromDiscriminant("Frequency")
          )
        : [],
      other: { type: "Silence" },
    });
    setOpenEditor(true);
  };

  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Audio table manager"
    >
      <h2>Audio Table Manager</h2>
      {audioTable === null ? (
        <>
          <button onClick={createAudioTable}>Create Audio table</button>
          <LoadButton
            text="Load audio table from file"
            onLoad={client.loadAudioTable}
          />
        </>
      ) : (
        <>
          <Dialog
            modal={true}
            open={openEditor}
            setOpen={setOpenEditor}
            openText="Edit"
          >
            <AudioTableEditor
              audioTable={audioTable}
              colourTable={colourTable}
              onSave={(newTable) => {
                client.setAudioTable(newTable);
                setOpenEditor(false);
              }}
            />
          </Dialog>
          <SaveButton text="Export" onSave={client.saveAudioTable} />
          <button
            onClick={() => {
              if (confirm("Are you sure you want to delete this audio table?"))
                client.setAudioTable(null);
            }}
          >
            Delete audio table
          </button>
        </>
      )}
    </Dialog>
  );
};

const EscSounds = await client.getEscSounds();

type AudioTableEditorProps = {
  audioTable: AudioTable;
  colourTable: ColourTable | null;
  onSave: (table: AudioTable) => void;
};

const AudioTableEditor = ({
  audioTable: initialTable,
  colourTable,
  onSave,
}: AudioTableEditorProps) => {
  const [audioTable, setAudioTable] = useState(initialTable);
  useEffect(() => {
    setAudioTable(initialTable);
  }, [audioTable]);
  return (
    <div>
      <h3>Edit Audio table</h3>
      <div>
        Default sound / sound for unknown pixel values:
        <AudioTypeInput
          audioType={
            audioTable?.other ??
            getDefaultAudioTypeFromDiscriminant("Frequency")
          }
          setAudioType={(audioType) =>
            setAudioTable({ other: audioType, entries: audioTable.entries })
          }
        />
      </div>
      <ul>
        {audioTable?.entries.map((entry, index) => (
          <li>
            {index}:
            <AudioTypeInput
              audioType={entry}
              setAudioType={(audioType) => {
                const newEntries = audioTable.entries.slice();
                newEntries[index] = audioType;
                setAudioTable({ other: audioTable.other, entries: newEntries });
              }}
            />
          </li>
        ))}
      </ul>
      <button
        onClick={() =>
          setAudioTable({
            other: audioTable!.other,
            entries: [
              ...audioTable!.entries,
              getDefaultAudioTypeFromDiscriminant("Frequency"),
            ],
          })
        }
      >
        Add entry
      </button>
      <button onClick={() => onSave(audioTable)}>Save</button>
    </div>
  );
};

const AudioTypeSelector = bindedSelectorFactory(await client.getAudioTypes());

const getDefaultAudioTypeFromDiscriminant = (
  discriminant: AudioTypeDiscriminants
): AudioType => {
  switch (discriminant) {
    case "Silence":
    case "LinearMap":
      return { type: discriminant };
    case "Speak":
      return { type: discriminant, value: "" };
    case "Frequency":
      return { type: discriminant, value: 0 };
    case "EscSound":
      return { type: discriminant, value: 0 };
  }
};

type AudioTypeInputProps = {
  audioType: AudioType;
  setAudioType: (audioType: AudioType) => void;
};

const AudioTypeInput = ({ audioType, setAudioType }: AudioTypeInputProps) => {
  return (
    <>
      {" "}
      <AudioTypeSelector
        prompt="Sound for this pixel"
        binding={{
          value: audioType.type,
          setValue: (discriminant) =>
            setAudioType(getDefaultAudioTypeFromDiscriminant(discriminant)),
        }}
      />
      {audioType.type === "Frequency" ? (
        <NumberInput
          label="Frequency for this pixel"
          binding={{
            value: audioType.value,
            setValue: (value) => setAudioType({ type: "Frequency", value }),
          }}
        />
      ) : audioType.type === "Speak" ? (
        <TextInput
          label="Word or phrase to say for this pixel"
          binding={{
            value: audioType.value,
            setValue: (value) => setAudioType({ type: "Speak", value }),
          }}
        />
      ) : audioType.type === "EscSound" ? (
        <EscSoundSelector
          selected={audioType.value}
          setSound={(value) => setAudioType({ type: "EscSound", value })}
        />
      ) : (
        ""
      )}
    </>
  );
};

const resourcesPath = await Path.resourceDir();
const audioPath = await Path.join(resourcesPath, "esc-50/audio");

type EscSoundSelectorProps = {
  selected: number;
  setSound: (index: number) => void;
};

const EscSoundSelector = ({ selected, setSound }: EscSoundSelectorProps) => {
  const selectedSound = Object.values(EscSounds)
    .flat()
    .find((sound) => sound.index === selected)!;

  const [category, setCategory] = useState(selectedSound.category);
  const indexInCategory = EscSounds[category].indexOf(selectedSound);
  const playAudio = async (name: string) => {
    const path = await Path.join(audioPath, name);
    console.log(path);
    const src = convertFileSrc(path);
    console.log(src);
    const audio = new Audio(src);
    return audio.play();
  };
  console.log(selectedSound);
  return (
    <div>
      <OptionPicker
        prompt="Category"
        options={Object.keys(EscSounds)}
        selectedOption={category}
        setOption={setCategory}
        emptyText="This shouldn't be empty"
      />
      <IndexedOptionPicker
        prompt="Sound"
        options={EscSounds[category].map((sound) => sound.filename)}
        index={indexInCategory === -1 ? null : indexInCategory}
        setIndex={(index) => {
          const sound = EscSounds[category][index];
          playAudio(sound.filename);
          setSound(sound.index);
        }}
        emptyText="This shouldn't be empty"
      />
    </div>
  );
};
