import { FeatureCollection } from "geojson";
import { CoordinateBox, VectorSettings } from "touch-device";

export type VectorLayerLocator =
  | { type: "url"; url: string }
  | { type: "file"; file: File }
  | { type: "text"; text: string };

export type RasterLayerLocator =
  | { type: "url"; url: string }
  | { type: "file"; file: File };

export type InitialVectorData = {
  features: FeatureCollection;
  name: string;
  displaySettings: VectorSettings | null;
};

export type InitialData = {
  vector: InitialVectorData[];
  raster: ArrayBuffer | null;
  audioTable: string | null;
  voice: SpeechSynthesisVoice;
  bounds: CoordinateBox;
};
