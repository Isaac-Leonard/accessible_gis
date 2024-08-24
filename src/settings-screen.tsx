import { client } from "./api";
import { AudioSettings, GlobalSettings } from "./bindings";
import { selectorFactory } from "./option-picker";
import { getterSetterFactory } from "./utils";

type SettingsScreenProps = { settings: GlobalSettings };

export const SettingsScreen = ({ settings }: SettingsScreenProps) => {
  return (
    <div>
      <h1>Settings</h1>
      <h2>General Settings</h2>{" "}
      <label>
        Display first raster by default
        <input
          type="checkbox"
          checked={settings.display_first_raster}
          onChange={(e) =>
            client.setShowFirstRasterByDefault(e.currentTarget.checked)
          }
        />
      </label>
      <label>
        Show towns by default
        <input
          type="checkbox"
          checked={settings.show_towns_by_default}
          onChange={(e) =>
            client.setShowTownsByDefault(e.currentTarget.checked)
          }
        />
      </label>
      <label>
        Show countries by default
        <input
          type="checkbox"
          checked={settings.show_countries_by_default}
          onChange={(e) =>
            client.setShowCountriesByDefault(e.currentTarget.checked)
          }
        />
      </label>
      <label>
        Enable OCR by default for rasters that use the GDAL rendering method
        <input
          type="checkbox"
          checked={settings.default_ocr_for_gdal}
          onChange={(e) => client.setDefaultOcrForGdal(e.currentTarget.checked)}
        />
      </label>
      <AudioSettingsScreen
        settings={settings.audio}
        setSettings={client.setDefaultAudio}
      />{" "}
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
  const getterSetter = getterSetterFactory(settings, setSettings);
  return (
    <div>
      <h2>Audio Settings</h2>
      <AudioIndicatorSelector
        prompt="Sound when touching areas with no data"
        {...getterSetter.getSet("no_data_value_sound", "selectedOption")}
      />
      <AudioIndicatorSelector
        prompt="Sound when touching border of image"
        {...getterSetter.getSet("border_sound", "selectedOption")}
      />
    </div>
  );
};

const AudioIndicatorSelector = selectorFactory(
  await client.getAudioIndicators()
);
