import {
  FeatureInfo,
  VectorScreenData,
  VectorScreenMetadata,
} from "../bindings";
import { GeometryViewer } from "./geometry";
import { OptionPicker } from "../option-picker";
import { client } from "../api";
import { Dialog, useDialog } from "../dialog";
import { ReprojectionDialog } from "../reprojection-dialog";
import { FeaturePicker } from "./feature-picker";
import { FeatureCoppierDialog } from "./feature-copier";
import { LayerSimplifierDialog } from "./layer_simplifier";
import { GdalMetadataViewer } from "../raster-navigator";
import { FieldsTable } from "./attribute-table";
import { DatasetEditor, EditDatasetButton } from "./dataset-editor";
import { LayerSettingsDialog } from "./settings";

type VectorLayerProp = {
  layer: VectorScreenData;
};

export const VectorNavigator = ({ layer }: VectorLayerProp) => {
  return (
    <div>
      <MetadataDialog metadata={layer.metadata} />
      <ReprojectionDialog />
      <FeatureCoppierDialog layer={layer} />
      <LayerSimplifierDialog />
      <button
        onClick={(_) =>
          client
            .getLandformDescription()
            .then((description) => alert(description))
        }
      >
        View Landform Description
      </button>
      {layer.editable ? <DatasetEditor layer={layer} /> : <EditDatasetButton />}
      <button onClick={client.setDisplayVector}>
        {layer.info.display ? "Remove from screen" : "Show on screen"}
      </button>
      {layer.info.display ? (
        <button onClick={client.focusDataset}>Focus Layer</button>
      ) : null}
      <LayerSettingsDialog layer={layer} />
      <FeaturePicker layer={layer} />
      <FeatureViewer layer={layer} />
    </div>
  );
};

const MetadataDialog = ({ metadata }: { metadata: VectorScreenMetadata }) => {
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      openText="Layer Metadata"
      modal={true}
      open={open}
      setOpen={setOpen}
    >
      <h3>projection</h3>
      <div>srs: {metadata.srs}</div>
      <GdalMetadataViewer metadata={metadata.other} />
    </Dialog>
  );
};

type CurrentFeatureViewerProps = {
  info: FeatureInfo;
  layer: VectorScreenData;
};

const CurrentFeatureViewer = ({ info, layer }: CurrentFeatureViewerProps) => {
  const { open: pointsOpen, setOpen: setPointsOpen } = useDialog();
  const { open: fieldsOpen, setOpen: setFieldsOpen } = useDialog();
  return (
    <div>
      <Dialog
        modal={true}
        open={pointsOpen}
        setOpen={setPointsOpen}
        openText="Examine points"
      >
        {info.geometry !== null ? (
          <GeometryViewer geometry={info.geometry} srs={layer.srs} />
        ) : (
          <div>No Geometry</div>
        )}
      </Dialog>
      <Dialog
        modal={true}
        open={fieldsOpen}
        setOpen={setFieldsOpen}
        openText="Open fields table"
      >
        <FieldsTable
          fields={info.fields}
          preferedDisplayField={
            layer.info.touch_device_settings!.audio.prefered_label_field
          }
        />
      </Dialog>
    </div>
  );
};

const FeatureViewer = ({ layer }: VectorLayerProp) => {
  return layer.feature !== null ? (
    <CurrentFeatureViewer info={layer.feature} layer={layer} />
  ) : (
    <div>This layer has no features yet, maybe create some?</div>
  );
};
