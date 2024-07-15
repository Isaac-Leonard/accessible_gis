import { VectorNavigator } from "./vector-navigator";
import { RasterNavigator } from "./raster-navigator";
import { Info, LayerDescriptor, UiPayload } from "./bindings";
import { IndexedOptionPicker } from "./option-picker";
import { client, state } from "./api";
import { OpenDatasetDialog } from "./open-screen";

export const MainScreen = () => {
  const value = state.value;
  return value.type === "Uninitialised" ? (
    <OpenDatasetDialog />
  ) : (
    <InitialisedStateScreen state={value} />
  );
};

const InitialisedStateScreen = ({
  state,
}: {
  state: Extract<UiPayload, { type: "Initialised" }>;
}) => {
  const layersInfo = state.layers;
  const foundIndex = layersInfo.findIndex(
    (layer) =>
      layer.type === state.info?.type &&
      layer.dataset === state.info.dataset_index &&
      layer.band.index === state.info.layer_index
  );
  const selectedLayerIndex = foundIndex === -1 ? null : foundIndex;
  return (
    <div className="container">
      <OpenDatasetDialog />
      <LayerSelector layers={state.layers} selectedIndex={selectedLayerIndex} />
      <LayerView layer={state.info} />{" "}
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
        setIndex={async (index) => {
          const { dataset, band } = layers[index];
          // TODO: These should probably be put into a single function
          client.setDatasetIndex(dataset);
          client.setLayerIndex(band);
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

function CoordinateExplorer({ layer }: { layer: Info }) {
  return layer.type === "Raster" ? (
    <RasterNavigator layer={layer} />
  ) : (
    <VectorNavigator layer={layer} />
  );
}

const Metadata = ({ layer }: { layer: Info }) => {
  return (
    <div>
      {" "}
      <div>SRS: {layer.srs}</div>
      <div>Projection: {layer.projection}</div>
    </div>
  );
};

const CurrentLayerView = ({ layer }: { layer: Info }) => {
  return (
    <div>
      <Metadata layer={layer} />
      <CoordinateExplorer layer={layer} />
    </div>
  );
};

const LayerView = ({ layer }: { layer: Info | null }) => {
  return layer !== null ? (
    <CurrentLayerView layer={layer} />
  ) : (
    <div>No layers selected</div>
  );
};
