import { VectorNavigator } from "./vector/";
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
import { LoadButton, SaveButton } from "./save-button";

export const MainScreen = ({ state }: { state: ProjectScreen }) => {
  return state.type === "NotLoaded" ? (
    <ProjectActions />
  ) : (
    <LoadedProjectScreen state={state} />
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

  // Variables for project actions Popup
  const { open, setOpen } = useDialog();

  return (
    <div className="container" onKeyDown={keyHandler}>
      <Dialog
        modal={true}
        open={open}
        setOpen={setOpen}
        openText="Project actions"
      >
        <ProjectActions onAction={() => setOpen(false)} />
      </Dialog>
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
      {selectedIndex !== null ? (
        <button onClick={client.removeDataset}>Remove Dataset</button>
      ) : null}
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

// onAction is here so we can close the dialog this is in when a project is already loaded
const ProjectActions = ({ onAction }: { onAction?: () => void }) => {
  const newProject = (name: string) => {
    client.createProject(name);
    onAction?.();
  };

  const loadProject = (file: string) => {
    client.loadProject(file);
    onAction?.();
  };

  return (
    <div>
      <SaveButton
        text="New Project"
        onSave={newProject}
        prompt="Project location"
        defaultPath="accessible_gis_project.json"
      />
      <LoadButton
        prompt="Load project from where"
        onLoad={loadProject}
        text="              Open Project"
      />
    </div>
  );
};
