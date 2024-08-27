import { client } from "./api";
import { AudioSettings, GlobalSettings } from "./bindings";
import { bindedSelectorFactory } from "./option-picker";
import { Checkbox, useBindedObjectProperties } from "./binded-input";

type SettingsScreenProps = { settings: GlobalSettings };

export const SettingsScreen = ({ settings }: SettingsScreenProps) => {
  const boundSettings = useBindedObjectProperties(settings, client.setSettings);
  return (
    <div>
      <h1>Settings</h1>
      <h2>General Settings</h2>{" "}
      <Checkbox
        label="Display first raster by default"
        binding={boundSettings.display_first_raster}
      />
      <Checkbox
        label="Show towns by default"
        binding={boundSettings.show_towns_by_default}
      />
      <Checkbox
        label="Show countries by default"
        binding={boundSettings.show_countries_by_default}
      />
      <Checkbox
        label="Enable OCR by default for rasters that use the GDAL rendering method"
        binding={boundSettings.default_ocr_for_gdal}
      />
      <AudioSettingsScreen
        settings={boundSettings.audio.value}
        setSettings={boundSettings.audio.setValue}
      />
    </div>
  );
};

type AudioSettingsScreenProps = {
  settings: AudioSettings;
  setSettings: (settings: AudioSettings) => void;
};

const AudioSettingsScreen = ({
  settings,
  setSettings,
}: AudioSettingsScreenProps) => {
  const getterSetter = useBindedObjectProperties(settings, setSettings);
  return (
    <div>
      <h2>Audio Settings</h2>
      <AudioIndicatorSelector
        prompt="Sound when touching areas with no data"
        binding={getterSetter.no_data_value_sound}
      />
      <AudioIndicatorSelector
        prompt="Sound when touching border of image"
        binding={getterSetter.border_sound}
      />
    </div>
  );
};

const AudioIndicatorSelector = bindedSelectorFactory(
  await client.getAudioIndicators()
);
