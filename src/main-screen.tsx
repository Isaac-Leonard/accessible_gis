import { Suspense, useDeferredValue, useState } from "react";
import { VectorNavigator } from "./vector-navigator";
import { RasterNavigator } from "./raster-navigator";
import { LayerDescriptor } from "./bindings";
import { invoke } from "@tauri-apps/api";
import { suspend } from "suspend-react";
import { openFile } from "./files";

// const audioCtx = new AudioContext();

const loadFile = async (name: string): Promise<string> => {
  return await invoke<string>("load_file", { name }).catch(
    (e) => e as unknown as string
  );
};

export const MainScreen = () => {
  const [url, setUrl] = useState<string>("");
  const [fileStatus, setFileStatus] = useState<null | string>(null);
  const layersInfo = suspend(
    async () =>
      fileStatus?.startsWith("Success")
        ? invoke<LayerDescriptor[]>("get_app_info")
        : [],
    [fileStatus]
  );

  let [selectedLayerIndex, setSelectedLayerIndex] = useState(0);
  selectedLayerIndex = useDeferredValue(selectedLayerIndex);

  const selectedLayer: LayerDescriptor | null =
    layersInfo[selectedLayerIndex] ?? null;
  async function load() {
    const file = await openFile();
    if (file !== null) {
      setFileStatus(await loadFile(file));
    }
  }
  return (
    <div className="container">
      <button onClick={load}>Open</button>
      <label>
        Load from url:
        <input value={url} onChange={(e) => setUrl(e.target.value)} />
      </label>
      <button onClick={async () => setFileStatus(await loadFile(url))}>
        Load url
      </button>
      <p>{fileStatus ?? "unloaded"}</p>
      <LayerSelector
        layers={layersInfo}
        selectedLayer={selectedLayerIndex}
        setLayer={setSelectedLayerIndex}
      />
      <Suspense fallback="Loading..">
        {selectedLayer !== null ? (
          <div>
            <div>SRS: {selectedLayer.srs}</div>
            <div>Projection: {selectedLayer.projection}</div>
            <CoordinateExplorer layer={selectedLayer} />
          </div>
        ) : null}
      </Suspense>
    </div>
  );
};

function LayerSelector({
  selectedLayer,
  layers,
  setLayer,
}: {
  selectedLayer: number;
  setLayer: (layer: number) => void;
  layers: LayerDescriptor[];
}) {
  return (
    <div>
      <select
        value={selectedLayer}
        onChange={(e) => setLayer(Number(e.target.value))}
      >
        {layers.map((layer, index) => (
          <option value={index.toString()} key={index.toString()}>
            {layer.dataset_file.split("/").pop()}:{layer.type}
          </option>
        ))}
      </select>
    </div>
  );
}

function CoordinateExplorer({ layer }: { layer: LayerDescriptor }) {
  if (layer.type === "Raster") {
    return <RasterNavigator layer={layer} />;
  } else {
    return (
      <Suspense fallback="loading">
        <VectorNavigator layer={layer} />
      </Suspense>
    );
  }
}
