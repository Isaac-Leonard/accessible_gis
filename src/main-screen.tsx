import { VectorNavigator } from "./vector-navigator";
import { RasterNavigator } from "./raster-navigator";
import { LayerDescriptor, LayerScreen, LayerScreenInfo } from "./bindings";
import { IndexedOptionPicker } from "./option-picker";
import { client } from "./api";
import { OpenDatasetDialog } from "./open-screen";

export const MainScreen = ({ state }: { state: LayerScreen }) => {
  const layersInfo = state.layers;
  const foundIndex = layersInfo.findIndex(
    (layer) =>
      layer.type === state.layer_info?.type &&
      layer.dataset === state.layer_info.dataset_index &&
      layer.index === state.layer_info.layer_index
  );
  const selectedLayerIndex = foundIndex === -1 ? null : foundIndex;
  return (
    <div className="container">
      <OpenDatasetDialog />
      <LayerSelector layers={state.layers} selectedIndex={selectedLayerIndex} />
      <LayerView layer={state.layer_info} />
    </div>
  );
};

type LayerSelectorProps = {
  layers: LayerDescriptor[];
  selectedIndex: number | null;
};

function LayerSelector({ layers, selectedIndex }: LayerSelectorProps) {
  return (
    <div>
      <IndexedOptionPicker
        index={selectedIndex}
        setIndex={async (layer_index) => {
          const { dataset, type, index } = layers[layer_index];
          // TODO: These should probably be put into a single function
          client.setDatasetIndex(dataset);
          client.setLayerIndex({ type, index });
        }}
        options={layers.map(
          (layer) => `${layer.dataset_file.split("/").pop()}: ${layer.type}`
        )}
        emptyText="No layers loaded"
        prompt="Select layer"
      ></IndexedOptionPicker>
    </div>
  );
}

function CoordinateExplorer({ layer }: { layer: LayerScreenInfo }) {
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

const CurrentLayerView = ({ layer }: { layer: LayerScreenInfo }) => {
  return (
    <div>
      <Metadata layer={layer} />
      <CoordinateExplorer layer={layer} />
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
