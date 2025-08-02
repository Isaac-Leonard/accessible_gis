import { VectorNavigator } from "./vector-navigator";
import { RasterNavigator } from "./raster-navigator";
import {
  DatasetLayerIndex,
  LayerDescriptor,
  LayerScreenInfo,
  ProjectScreen,
  ProjectScreenInfo,
} from "./bindings";
import { IndexedOptionPicker } from "./option-picker";
import { client } from "./api";
import { OpenDatasetDialog } from "./open-screen";
import { Dialog, useDialog } from "./dialog";
import { newProject, openFile } from "./files";
import { LayerScreenContext } from "./context";

export const MainScreen = ({ state }: { state: ProjectScreen }) => {
  return state.type === "NotLoaded" ? (
    <div>
      <button onClick={newProject}>New Project</button>
      <button
        onClick={() =>
          openFile("Load project from where").then((file) => {
            if (file !== null) {
              client.loadProject(file);
            }
          })
        }
      >
        Open Project
      </button>
    </div>
  ) : (
    <LayerScreenContext.Provider value={state}>
      <LoadedProjectScreen state={state} />
    </LayerScreenContext.Provider>
  );
};

export const LoadedProjectScreen = ({
  state,
}: {
  state: ProjectScreenInfo;
}) => {
  const layersInfo = state.layers;
  const foundIndex = layersInfo.findIndex(
    (layer) =>
      layer.type === state.layer_info?.type &&
      layer.dataset === state.layer_info.dataset_index &&
      layer.index === state.layer_info.layer_index
  );
  const selectedLayerIndex = foundIndex === -1 ? null : foundIndex;

  const setLayerIndex = async (layer_index: number) => {
    const { dataset, type, index } = layersInfo[layer_index];
    // TODO: These should probably be put into a single function
    client.setDatasetIndex(dataset);
    client.setLayerIndex({ type, index });
  };

  const showPreviousLayer = () => {
    if (selectedLayerIndex !== null && selectedLayerIndex !== 0) {
      setLayerIndex(selectedLayerIndex - 1);
    }
  };

  const showNextLayer = () => {
    if (
      selectedLayerIndex !== null &&
      selectedLayerIndex !== layersInfo.length - 1
    ) {
      setLayerIndex(selectedLayerIndex + 1);
    }
  };

  const keyHandler = (e: KeyboardEvent) => {
    if (e.ctrlKey) {
      switch (e.key) {
        case "p":
          e.preventDefault();
          showPreviousLayer();
          break;
        case "n":
          e.preventDefault();
          showNextLayer();
          break;
      }
    }
  };

  return (
    <div className="container" onKeyDown={keyHandler}>
      <OpenDatasetDialog />
      <IpDialog ip={state.ip} />
      <LayerSelector
        layers={state.layers}
        selectedIndex={selectedLayerIndex}
        setLayer={async (layer_index) => {
          // TODO: These should probably be put into a single function
          client.setDatasetIndex(layer_index.dataset);
          client.setLayerIndex(layer_index.layer);
        }}
      />
      <LayerView layer={state.layer_info} />
    </div>
  );
};

type LayerSelectorProps = {
  layers: LayerDescriptor[];
  selectedIndex: number | null;
  setLayer: (layer: DatasetLayerIndex) => void;
};

export function LayerSelector({
  layers,
  selectedIndex,
  setLayer,
}: LayerSelectorProps) {
  return (
    <div>
      <IndexedOptionPicker
        index={selectedIndex}
        setIndex={(index) =>
          setLayer({
            dataset: layers[index].dataset,
            layer: { type: layers[index].type, index: layers[index].index },
          })
        }
        options={layers.map(
          (layer) => `${layer.dataset_file.split("/").pop()}: ${layer.type}`
        )}
        emptyText="No layers loaded"
        prompt="Select layer"
      ></IndexedOptionPicker>
    </div>
  );
}

function InnerLayerView({ layer }: { layer: LayerScreenInfo }) {
  return layer.type === "Raster" ? (
    <RasterNavigator layer={layer} />
  ) : (
    <VectorNavigator layer={layer} />
  );
}

const Metadata = ({ layer }: { layer: LayerScreenInfo }) => {
  return (
    <div>
      {" "}
      <div>SRS: {layer.srs}</div>
    </div>
  );
};

const IpDialog = ({ ip }: { ip: string }) => {
  const { open, setOpen, innerRef } = useDialog<HTMLParagraphElement>();
  return (
    <Dialog
      open={open}
      setOpen={setOpen}
      openText="Check current IP to connect a touch device"
    >
      <p ref={innerRef}>{ip}</p>
      <button onClick={() => setOpen(false)}>Close</button>
    </Dialog>
  );
};

const CurrentLayerView = ({ layer }: { layer: LayerScreenInfo }) => {
  return (
    <div>
      <Metadata layer={layer} />
      <InnerLayerView layer={layer} />
    </div>
  );
};

const LayerView = ({ layer }: { layer: LayerScreenInfo | null }) => {
  return layer !== null ? (
    <CurrentLayerView layer={layer} />
  ) : (
    <div>No layers selected</div>
  );
};
