import { save } from "@tauri-apps/plugin-dialog";
import { OptionPicker } from "./option-picker";
import { useSignal } from "@preact/signals";
import { useState } from "preact/hooks";
import { client } from "./api";
const getFileName = () => save();

export const NewDatasetScreen = ({ drivers }: { drivers: string[] }) => {
  const driver = useSignal(drivers[0]);
  const [vector, setVector] = useState(true);
  const [{ width, height }, setSize] = useState({ width: 1000, height: 1000 });
  const [file, setFile] = useState<string | null>(null);

  return (
    <div>
      <OptionPicker
        options={drivers}
        selectedOption={driver.value}
        setOption={(v) => {
          driver.value = v;
        }}
        prompt="Driver to create the dataset with:"
        emptyText="This should not be empty"
      />
      <button
        onClick={() =>
          getFileName().then((file) => {
            console.log("here" + file);
            if (file !== null) {
              setFile(file);
            }
          })
        }
      >
        Save dataset to {file !== null ? file : ""}
      </button>
      <button onClick={() => setVector(!vector)}>
        Create {vector ? "vector" : "raster"} dataset
      </button>
      {!vector ? (
        <div>
          <label>
            Width:
            <input
              value={width}
              onChange={(e) =>
                setSize({ width: Number(e.currentTarget.value), height })
              }
            />
          </label>
          <label>
            Height
            <input
              value={height}
              onChange={(e) =>
                setSize({ width: Number(e.currentTarget.value), height })
              }
            />
          </label>
        </div>
      ) : null}
      <button
        disabled={file === null}
        onClick={() => client.createNewDataset(driver.value, file!)}
      >
        Create
      </button>
    </div>
  );
};
