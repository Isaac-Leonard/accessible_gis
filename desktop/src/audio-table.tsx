import { useEffect, useState } from "preact/hooks";
import { client } from "./api";
import {
  AudioTable,
  AudioType,
  AudioTypeDiscriminants,
  ColourTable,
} from "./bindings";
import { Dialog, useDialog } from "./dialog";
import { LoadButton, SaveButton } from "./save-button";
import {
  bindedSelectorFactory,
  IndexedOptionPicker,
  OptionPicker,
} from "./option-picker";
import { NumberInput, Input as TextInput } from "./binded-input";
import * as Path from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";

type AudioTableManagerProps = {
  audioTable: AudioTable | null;
  colourTable: ColourTable | null;
};

export const AudioTableManager = ({
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
