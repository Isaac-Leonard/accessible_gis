// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/

export const commands = {
  async loadFile(name: string): Promise<Result<null, string>> {
    try {
      return { status: "ok", data: await TAURI_INVOKE("load_file", { name }) };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async getAppInfo(): Promise<UiScreen> {
    return await TAURI_INVOKE("get_app_info");
  },
  async getBandSizes(): Promise<RasterSize[]> {
    return await TAURI_INVOKE("get_band_sizes");
  },
  async getValueAtPoint(point: Point): Promise<number | null> {
    return await TAURI_INVOKE("get_value_at_point", { point });
  },
  async getPointOfMaxValue(): Promise<Point | null> {
    return await TAURI_INVOKE("get_point_of_max_value");
  },
  async getPointOfMinValue(): Promise<Point | null> {
    return await TAURI_INVOKE("get_point_of_min_value");
  },
  async getPolygonsAroundPoint(
    point: Point,
    layer: number
  ): Promise<PolygonInfo[]> {
    return await TAURI_INVOKE("get_polygons_around_point", { point, layer });
  },
  async describeLine(
    line: LineString,
    srs: string | null,
    distance: number,
    towns: number
  ): Promise<LineDescription> {
    return await TAURI_INVOKE("describe_line", { line, srs, distance, towns });
  },
  async describePolygon(polygon: Polygon): Promise<string> {
    return await TAURI_INVOKE("describe_polygon", { polygon });
  },
  async pointInCountry(point: Point): Promise<DistanceFromBoarder | null> {
    return await TAURI_INVOKE("point_in_country", { point });
  },
  async nearestTown(point: Point): Promise<DistanceFromBoarder | null> {
    return await TAURI_INVOKE("nearest_town", { point });
  },
  async theissenPolygonsCalculation(
    records: ThiessenPolygonRecord[],
    srs: string
  ): Promise<number[]> {
    return await TAURI_INVOKE("theissen_polygons_calculation", {
      records,
      srs,
    });
  },
  async theissenPolygons(points: MultiPoint, srs: string): Promise<Polygon[]> {
    return await TAURI_INVOKE("theissen_polygons", { points, srs });
  },
  async getCsv(file: string): Promise<string[][]> {
    return await TAURI_INVOKE("get_csv", { file });
  },
  async theissenPolygonsToFile(
    points: MultiPoint,
    srs: string,
    file: string
  ): Promise<void> {
    await TAURI_INVOKE("theissen_polygons_to_file", { points, srs, file });
  },
  async setScreen(screen: Screen): Promise<void> {
    await TAURI_INVOKE("set_screen", { screen });
  },
  async setLayerIndex(index: LayerIndex): Promise<void> {
    await TAURI_INVOKE("set_layer_index", { index });
  },
  async setDatasetIndex(index: number): Promise<void> {
    await TAURI_INVOKE("set_dataset_index", { index });
  },
  async setFeatureIndex(index: number): Promise<Result<null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("set_feature_index", { index }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async createNewDataset(
    driverName: string,
    file: string
  ): Promise<Result<null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("create_new_dataset", { driverName, file }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async addFieldToSchema(
    name: string,
    fieldType: FieldType
  ): Promise<Result<null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("add_field_to_schema", { name, fieldType }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async editDataset(): Promise<Result<null, string>> {
    try {
      return { status: "ok", data: await TAURI_INVOKE("edit_dataset") };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async addFeatureToLayer(feature: FeatureInfo): Promise<Result<null, string>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("add_feature_to_layer", { feature }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async getImagePixels(): Promise<Result<number[], string>> {
    try {
      return { status: "ok", data: await TAURI_INVOKE("get_image_pixels") };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async setNameField(field: string): Promise<void> {
    await TAURI_INVOKE("set_name_field", { field });
  },
  async classifyCurrentRaster(
    dest: string,
    classifications: Classification[]
  ): Promise<void> {
    await TAURI_INVOKE("classify_current_raster", { dest, classifications });
  },
  async setSrs(srs: Srs): Promise<void> {
    await TAURI_INVOKE("set_srs", { srs });
  },
  async reprojectLayer(srs: Srs, name: string): Promise<void> {
    await TAURI_INVOKE("reproject_layer", { srs, name });
  },
  async copyFeatures(features: number[], name: string): Promise<void> {
    await TAURI_INVOKE("copy_features", { features, name });
  },
  async simplifyLayer(tolerance: number, name: string): Promise<void> {
    await TAURI_INVOKE("simplify_layer", { tolerance, name });
  },
  async calcSlope(name: string): Promise<void> {
    await TAURI_INVOKE("calc_slope", { name });
  },
  async calcAspect(name: string): Promise<void> {
    await TAURI_INVOKE("calc_aspect", { name });
  },
  async calcRoughness(name: string): Promise<void> {
    await TAURI_INVOKE("calc_roughness", { name });
  },
  async playAsSound(): Promise<void> {
    await TAURI_INVOKE("play_as_sound");
  },
  async playHistogram(): Promise<void> {
    await TAURI_INVOKE("play_histogram");
  },
  async generateCountsReport(name: string): Promise<void> {
    await TAURI_INVOKE("generate_counts_report", { name });
  },
  async openSettings(): Promise<void> {
    await TAURI_INVOKE("open_settings");
  },
  async setSettings(settings: GlobalSettings): Promise<void> {
    await TAURI_INVOKE("set_settings", { settings });
  },
  /**
   * This file is for commands that return static data such as names for options
   */
  async getRenderMethods(): Promise<RenderMethod[]> {
    return await TAURI_INVOKE("get_render_methods");
  },
  async getAudioIndicators(): Promise<AudioIndicator[]> {
    return await TAURI_INVOKE("get_audio_indicators");
  },
  async getWaveForms(): Promise<Waveform[]> {
    return await TAURI_INVOKE("get_wave_forms");
  },
  async setDisplayRaster(): Promise<void> {
    await TAURI_INVOKE("set_display_raster");
  },
  async setDisplayVector(): Promise<void> {
    await TAURI_INVOKE("set_display_vector");
  },
  async setCurrentOcr(enabled: boolean): Promise<void> {
    await TAURI_INVOKE("set_current_ocr", { enabled });
  },
  async setCurrentRenderMethod(renderMethod: RenderMethod): Promise<void> {
    await TAURI_INVOKE("set_current_render_method", { renderMethod });
  },
  async setCurrentAudioSettings(settings: AudioSettings): Promise<void> {
    await TAURI_INVOKE("set_current_audio_settings", { settings });
  },
  async focusRaster(): Promise<void> {
    await TAURI_INVOKE("focus_raster");
  },
  async setPreferedDisplayFields(fields: string[]): Promise<void> {
    await TAURI_INVOKE("set_prefered_display_fields", { fields });
  },
};

/** user-defined events **/

export const events = __makeEvents__<{
  messageEvent: MessageEvent;
}>({
  messageEvent: "message-event",
});

/** user-defined constants **/

/** user-defined types **/

export type AudioIndicator =
  | "Silence"
  | "MinFreq"
  | "MaxFreq"
  | "Verbal"
  | "Different";
export type AudioSettings = {
  min_freq: number;
  max_freq: number;
  volume: number;
  no_data_value_sound: AudioIndicator;
  border_sound: AudioIndicator;
  histogram: HistogramSettings;
  graph: RasterGraphSettings;
};
export type Classification = { min: number; max: number; target: number };
export type ClosedLineDescription = {
  x: number;
  y: number;
  perimeter: number;
  area: number;
  countries: string[];
  towns: string[];
  waviness: number;
  distances: number;
  number_of_points: number;
};
export type DistanceFromBoarder = { name: string; distance: number };
export type Duration = { secs: number; nanos: number };
export type FeatureIdentifier = { name: string | null; fid: number };
export type FeatureInfo = {
  fields: Field[];
  geometry: Geometry | null;
  fid: number | null;
};
export type Field = (
  | { type: "Integer"; value: number }
  | { type: "IntegerList"; value: number[] }
  | { type: "Integer64"; value: number }
  | { type: "Integer64List"; value: number[] }
  | { type: "String"; value: string }
  | { type: "StringList"; value: string[] }
  | { type: "Real"; value: number }
  | { type: "RealList"; value: number[] }
  | { type: "Date"; value: string }
  | { type: "DateTime"; value: string }
  | { type: "None" }
) & { name: string };
export type FieldSchema = { name: string; field_type: FieldType | null };
export type FieldType =
  /**
   * Simple 32 bit integer
   */
  | "OFTInteger"
  /**
   * List of 32 bit integers
   */
  | "OFTIntegerList"
  /**
   * Double precision floating point
   */
  | "OFTReal"
  /**
   * List of doubles
   */
  | "OFTRealList"
  /**
   * String of ascii chars
   */
  | "OFTString"
  /**
   * Array of strings
   */
  | "OFTStringList"
  /**
   * Deprecated
   */
  | "OFTWideString"
  /**
   * Deprecated
   */
  | "OFTWideStringList"
  /**
   * Raw binary data
   */
  | "OFTBinary"
  /**
   * Date
   */
  | "OFTDate"
  /**
   * Time
   */
  | "OFTTime"
  /**
   * Date and time
   */
  | "OFTDateTime"
  /**
   * Single 64 bit integer
   */
  | "OFTInteger64"
  /**
   * List of 64 bit integers
   */
  | "OFTInteger64List";
export type Geometry =
  | ({ type: "Point" } & Point)
  | ({ type: "Line" } & Line)
  | ({ type: "LineString" } & LineString)
  | ({ type: "Polygon" } & Polygon)
  | ({ type: "MultiPoint" } & MultiPoint)
  | ({ type: "MultiLineString" } & MultiLineString)
  | ({ type: "MultiPolygon" } & MultiPolygon)
  | ({ type: "GeometryCollection" } & GeometryCollection);
export type GeometryCollection = { geometries: Geometry[] };
export type GlobalSettings = {
  show_towns_by_default: boolean;
  show_countries_by_default: boolean;
  display_first_raster: boolean;
  default_ocr_for_gdal: boolean;
  default_rendering_method_for_images: RenderMethod;
  audio: AudioSettings;
};
export type HistogramSettings = {
  /**
   * The length the histogram should play for in milliseconds
   */
  duration: number;
  min_freq: number;
  max_freq: number;
};
export type LayerDescriptor = (
  | { type: "Vector"; index: number }
  | { type: "Raster"; index: number }
) & { dataset: number; dataset_file: string };
export type LayerIndex =
  | { type: "Vector"; index: number }
  | { type: "Raster"; index: number };
export type LayerScreen = {
  layers: LayerDescriptor[];
  layer_info: LayerScreenInfo | null;
  ip: string;
  prefered_display_fields: string[];
};
export type LayerScreenInfo =
  | ({ type: "Vector" } & VectorScreenData)
  | ({ type: "Raster" } & RasterScreenData);
export type Line = { start: Point; end: Point };
export type LineDescription =
  | ({ type: "Closed" } & ClosedLineDescription)
  | ({ type: "Open" } & OpenLineDescription);
export type LineString = { points: Point[] };
export type MessageEvent = null;
export type MultiLineString = { lines: LineString[] };
export type MultiPoint = { points: Point[] };
export type MultiPolygon = { polygons: Polygon[] };
export type NewDatasetScreenData = { drivers: string[] };
export type OpenLineDescription = {
  x: number;
  y: number;
  length: number;
  end_to_end_distance: number;
  angular_sum: number;
  countries: string[];
  towns: string[];
  waviness: number;
  distances: number;
  number_of_points: number;
};
export type Point = { x: number; y: number };
export type Polygon = { exterior: LineString; interior: LineString[] };
export type PolygonInfo = { area: number; fields: Field[] };
export type RasterGraphSettings = {
  /**
   * The length the histogram should play for in milliseconds
   */
  row_duration: Duration;
  min_freq: number;
  max_freq: number;
  rows: number;
  cols: number;
  classified: boolean;
  wave: Waveform;
  min_value: number | null;
  max_value: number | null;
};
export type RasterScreenData = {
  layer_index: number;
  dataset_index: number;
  cols: number;
  rows: number;
  srs: string | null;
  display: boolean;
  render_method: RenderMethod;
  ocr: boolean;
  audio_settings: AudioSettings;
};
export type RasterSize = { width: number; length: number; bands: number };
export type RenderMethod =
  /**
   * Try to use native browser image rendering or fall back to ImageJS
   */
  | "Image"
  /**
   * Render pure raster values mapped to 256 grey scale
   */
  | "GDAL";
export type Screen = "Main" | "NewDataset" | "Settings";
export type Srs =
  | { type: "Proj"; value: string }
  | { type: "Wkt"; value: string }
  | { type: "Esri"; value: string }
  | { type: "Epsg"; value: number };
export type ThiessenPolygonRecord = {
  point: Point;
  file: string;
  start_line: number;
  column: number;
};
export type UiScreen =
  | ({ name: "Layers" } & LayerScreen)
  | { name: "ThiessenPolygons" }
  | ({ name: "NewDataset" } & NewDatasetScreenData)
  | ({ name: "Settings" } & GlobalSettings);
export type VectorScreenData = {
  field_schema: FieldSchema[];
  features: FeatureIdentifier[];
  feature: FeatureInfo | null;
  srs: string | null;
  editable: boolean;
  layer_index: number;
  dataset_index: number;
  display: boolean;
  name_field: string | null;
};
export type Waveform = "Sine" | "Square" | "Triangle" | "Sawtooth";

/** tauri-specta globals **/

import {
  invoke as TAURI_INVOKE,
  Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: null extends T
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
  mappings: Record<keyof T, string>
) {
  return new Proxy(
    {} as unknown as {
      [K in keyof T]: __EventObj__<T[K]> & {
        (handle: __WebviewWindow__): __EventObj__<T[K]>;
      };
    },
    {
      get: (_, event) => {
        const name = mappings[event as keyof T];

        return new Proxy((() => {}) as any, {
          apply: (_, __, [window]: [__WebviewWindow__]) => ({
            listen: (arg: any) => window.listen(name, arg),
            once: (arg: any) => window.once(name, arg),
            emit: (arg: any) => window.emit(name, arg),
          }),
          get: (_, command: keyof __EventObj__<any>) => {
            switch (command) {
              case "listen":
                return (arg: any) => TAURI_API_EVENT.listen(name, arg);
              case "once":
                return (arg: any) => TAURI_API_EVENT.once(name, arg);
              case "emit":
                return (arg: any) => TAURI_API_EVENT.emit(name, arg);
            }
          },
        });
      },
    }
  );
}
