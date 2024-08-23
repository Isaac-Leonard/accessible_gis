import { client } from "./api";
import { AudioIndicator, AudioSettings, GlobalSettings } from "./bindings";
import { OptionPicker } from "./option-picker";

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
    </div>
  );
};

type AudioSettingsScreenProps = { settings: AudioSettings };

const AudioSettingsScreen = ({ settings }: AudioSettingsScreenProps) => {
  return (
    <div>
      <AudioIndicatorSelector selectedOption={settings.no_data_value_sound} />
      <AudioIndicatorSelector selectedOption={settings.border_sound} />
    </div>
  );
};

const audioIndicators = await client.getAudioIndicators();

type AudioIndicatorSelectorProps = {
  selectedOption: AudioIndicator;
  setIndicator: (indicator: AudioIndicator) => void;
};

const AudioIndicatorSelector = ({
  selectedOption,
  setIndicator,
}: AudioIndicatorSelectorProps) => {
  return (
    <OptionPicker
      options={audioIndicators}
      selectedOption={selectedOption}
      setOption={setIndicator}
      emptyText="Somethings wrong"
    />
  );
};
