import { client } from "../../api";
import {
  DesktopVectorOptions,
  SortOption,
  TouchDeviceVectorOptions,
  VectorScreenData,
} from "../../bindings";
import { Dialog, useDialog } from "../../dialog";
import { OptionPicker } from "../../option-picker";

type LayerSettingsDialogProps = {
  layer: VectorScreenData;
};

export const LayerSettingsDialog = ({ layer }: LayerSettingsDialogProps) => {
  const settings = layer.info;
  const { open, setOpen } = useDialog();
  return (
    <Dialog
      modal={true}
      open={open}
      setOpen={setOpen}
      openText="Layer settings"
    >
      <h3>Vector Layer Settings</h3>
      <DesktopSettings settings={settings.desktop_settings!} layer={layer} />
      <TouchDeviceSettings
        settings={settings.touch_device_settings!}
        layer={layer}
      />
    </Dialog>
  );
};

type DesktopSettingsProps = {
  settings: DesktopVectorOptions;
  layer: VectorScreenData;
};

const DesktopSettings = ({ settings, layer }: DesktopSettingsProps) => (
  <div>
    <h4>Desktop</h4>
    <NameAttributePicker
      layer={layer}
      nameAttribute={settings.primary_field_name}
    />
    <FeatureSorter sortBy={settings.sort_features_by} layer={layer} />
  </div>
);

type NameAttributePickerProps = {
  layer: VectorScreenData;
  nameAttribute: string | null;
};

const NameAttributePicker = ({
  layer,
  nameAttribute,
}: NameAttributePickerProps) => {
  const field_names = layer.field_schema.map((field) => field.name);
  return (
    <div>
      <OptionPicker
        options={field_names}
        selectedOption={nameAttribute}
        setOption={client.setNameField}
        emptyText=" This layer has no fields"
        prompt="Set name field"
      />
    </div>
  );
};

type FeatureSorterProps = {
  layer: VectorScreenData;
  sortBy: SortOption;
};

const FeatureSorter = ({ layer, sortBy }: FeatureSorterProps) => {
  const nameAttribute = layer.info.desktop_settings!.primary_field_name;
  const attributeNames = layer.field_schema.map((field) => field.name);
  const sortOptions = ["Default", "Field", "Area"] as const;

  const setSortBy = (sortType: (typeof sortOptions)[number]) => {
    if (sortType === "Field") {
      client.sortFeaturesBy({ option: sortType, settings: nameAttribute });
    } else {
      client.sortFeaturesBy({ option: sortType });
    }
  };

  return (
    <div>
      <OptionPicker
        prompt="Sort features by"
        options={sortOptions}
        selectedOption={sortBy.option}
        setOption={setSortBy}
        emptyText="This should not be empty"
      />
      {sortBy.option === "Field" ? (
        <OptionPicker
          options={attributeNames}
          selectedOption={sortBy.settings}
          setOption={(field) =>
            client.sortFeaturesBy({ option: "Field", settings: field })
          }
          emptyText=" This layer has no fields"
          prompt="Field to sort by"
        />
      ) : null}
    </div>
  );
};

type TouchDeviceSettingsProps = {
  layer: VectorScreenData;
  settings: TouchDeviceVectorOptions;
};

const TouchDeviceSettings = ({ settings }: TouchDeviceSettingsProps) => (
  <div>
    <h4>Touch Device</h4>
    <h5>Audio</h5>
    <button
      onClick={() => client.toggleAnnounceLeaving()}
      role="switch"
      aria-checked={settings?.audio.announce_leaving}
    >
      Toggle announcements when leaving polygons
    </button>
    <button
      onClick={() => client.toggleAnnounceGeometryTypes()}
      role="switch"
      aria-checked={settings?.audio.announce_geometry_type}
    >
      Toggle announcing types of geometries
    </button>
    <h5>Visual</h5>
    <button
      onClick={() => client.toggleLabels()}
      role="switch"
      aria-checked={settings?.visual.use_labels}
    >
      Toggle Auto Labels
    </button>
  </div>
);
