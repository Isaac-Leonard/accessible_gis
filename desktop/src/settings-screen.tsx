import { client } from "./api";
import { AudioSettings, GlobalSettings, RasterGraphSettings } from "./bindings";
import { bindedSelectorFactory } from "./option-picker";
import {
  Checkbox,
  NumberInput,
  useBindedObjectProperties,
} from "./binded-input";
import { H, Section } from "react-headings";

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

export const AudioSettingsScreen = ({
  settings,
  setSettings,
}: AudioSettingsScreenProps) => {
  const getterSetter = useBindedObjectProperties(settings, setSettings);
  return (
    <Section component={<H>Audio Settings</H>}>
      <AudioIndicatorSelector
        prompt="Sound when touching areas with no data"
        binding={getterSetter.no_data_value_sound}
      />
      <AudioIndicatorSelector
        prompt="Sound when touching border of image"
        binding={getterSetter.border_sound}
      />
      <RasterGraphSettingsScreen
        settings={getterSetter.graph.value}
        setSettings={getterSetter.graph.setValue}
      />
    </Section>
  );
};

const AudioIndicatorSelector = bindedSelectorFactory(
  await client.getAudioIndicators()
);

type RasterGraphSettingsScreenProps = {
  settings: RasterGraphSettings;
  setSettings: (settings: RasterGraphSettings) => void;
};

export const RasterGraphSettingsScreen = ({
  settings,
  setSettings,
}: RasterGraphSettingsScreenProps) => {
  const graphSettings = useBindedObjectProperties(settings, setSettings);
  return (
    <Section component={<H>Raster Graph Settings</H>}>
      <NumberInput
        label="Minimum frequency to play for the raster graph"
        binding={graphSettings.min_freq}
      />
      <NumberInput
        label="Maximum frequency to play for the raster graph"
        binding={graphSettings.max_freq}
      />
      <Checkbox
        label="Treat as classified data?"
        binding={graphSettings.classified}
      />
      <WaveformSelector
        prompt="Audio wave to use"
        binding={graphSettings.wave}
      />
      <NumberInput
        label="Shrink Horozontal resolution to x number of rows"
        binding={graphSettings.rows}
      />
      <NumberInput
        label="Shrink vertical resolution to y number of rows"
        binding={graphSettings.cols}
      />
    </Section>
  );
};

const WaveformSelector = bindedSelectorFactory(await client.getWaveForms());
