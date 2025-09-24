import { client } from "../../api";
import { Checkbox, Input, NumberInput } from "../../binded-input";
import {
  CssColour,
  CssColourDiscriminants,
  DesktopVectorOptions,
  RgbColour,
  SortOption,
  TouchDeviceVectorOptions,
  VectorScreenData,
} from "../../bindings";
import { Dialog, useDialog } from "../../dialog";
import { bindedSelectorFactory, OptionPicker } from "../../option-picker";

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
    <NumberInput
      label="Distance to line for announcement (km)"
      binding={{
        value: settings.audio!.distance_for_line_announcements!,
        setValue: client.setAudioLineWidth,
      }}
    />
    <NumberInput
      label="Distance to points for announcements (km)"
      binding={{
        value: settings.audio!.radius_for_point_announcements!,
        setValue: client.setAudioPointRadius,
      }}
    />
    <h5>Visual</h5>
    <ColourPicker
      prompt="Colour for vector outlines"
      colour={settings.visual.vector_line_colour}
      onDone={client.setVectorLineColour}
    />
    <NumberInput
      label="Vector line width (px)"
      binding={{
        value: settings.visual!.line_width!,
        setValue: client.setVectorLineWidth,
      }}
    />
    <NumberInput
      label="Vector point radius (px)"
      binding={{
        value: settings.visual!.point_radius!,
        setValue: client.setVectorPointRadius,
      }}
    />
    <h6>Labels</h6>
    <button
      onClick={() => client.toggleLabels()}
      role="switch"
      aria-checked={settings?.visual.labels?.enabled}
    >
      Toggle Auto Labels
    </button>
    <ColourPicker
      prompt="Colour for labels"
      colour={settings.visual.labels!.text_colour}
      onDone={client.setLabelColour}
    />
    <Input
      label="Label font"
      binding={{
        value: settings.visual.labels!.font,
        setValue: client.setLabelFont,
      }}
    />
    <NumberInput
      label="Text line width (px)"
      binding={{
        value: settings.visual.labels!.line_width!,
        setValue: client.setLabelLineWidth,
      }}
    />
    <Checkbox
      label="Fill label text"
      binding={{
        value: settings.visual.labels!.fill_text,
        setValue: () => client.toggleLabelFillText(),
      }}
    />{" "}
  </div>
);

const CssColourTypePicker = bindedSelectorFactory(
  await client.getCssColourTypes()
);

const colourFromDiscriminant = (
  discriminant: CssColourDiscriminants
): CssColour => {
  switch (discriminant) {
    case "Named":
      return { type: discriminant, value: "White" };
    case "Raw":
      return { type: discriminant, value: "" };
    case "Hex":
      return { type: discriminant, value: "ffffff" };
    case "Rgb":
      return { type: discriminant, value: [256, 256, 256] };
  }
};

const NamedColourPicker = bindedSelectorFactory(await client.getNamedColours());

type ColourPickerProps = {
  prompt: string;
  colour: CssColour;
  onDone: (colour: CssColour) => void;
};

export const ColourPicker = ({ colour, prompt, onDone }: ColourPickerProps) => {
  return (
    <div>
      {prompt}:
      <CssColourTypePicker
        prompt="Colour type"
        binding={{
          value: colour.type,
          setValue: (type) => onDone(colourFromDiscriminant(type)),
        }}
      />
      {colour.type === "Named" ? (
        <NamedColourPicker
          prompt="Name of colour"
          binding={{
            value: colour.value,
            setValue: (value) => onDone({ type: "Named", value }),
          }}
        />
      ) : colour.type === "Raw" ? (
        <Input
          label="Raw CSS colour input"
          binding={{
            value: colour.value,
            setValue: (value) => onDone({ type: "Raw", value }),
          }}
        />
      ) : colour.type === "Hex" ? (
        <Input
          label="Hex value"
          binding={{
            value: colour.value,
            setValue: (value) => onDone({ type: "Hex", value }),
          }}
        />
      ) : (
        <RgbColourEditor
          colour={colour.value}
          onDone={(colour) => onDone({ type: "Rgb", value: colour })}
        />
      )}
    </div>
  );
};

type RgbColourEditorProps = {
  colour: RgbColour;
  onDone: (colour: RgbColour) => void;
};

const RgbColourEditor = ({
  colour: [red, green, blue],
  onDone,
}: RgbColourEditorProps) => {
  return (
    <div>
      <NumberInput
        label="Red"
        binding={{ value: red, setValue: (red) => onDone([red, green, blue]) }}
      />
      <NumberInput
        label="Green"
        binding={{
          value: green,
          setValue: (green) => onDone([red, green, blue]),
        }}
      />
      <NumberInput
        label="Blue"
        binding={{
          value: blue,
          setValue: (blue) => onDone([red, green, blue]),
        }}
      />
    </div>
  );
};
