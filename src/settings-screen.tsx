import { client } from "./api";
import { GlobalSettings } from "./bindings";

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
    </div>
  );
};
