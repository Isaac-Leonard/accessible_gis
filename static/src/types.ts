export type RasterData =
  | { type: "UInt8"; data: number[] }
  | { type: "Int8"; data: number[] }
  | { type: "UInt16"; data: number[] }
  | { type: "Int16"; data: number[] }
  | { type: "UInt32"; data: number[] }
  | { type: "Int32"; data: number[] }
  | { type: "UInt64"; data: number[] }
  | { type: "Int64"; data: number[] }
  | { type: "Float32"; data: number[] }
  | { type: "Float64"; data: number[] };

export type FetchedImageData = {
  width: number;
  height: number;
  no_data_value: number | null;
} & RasterData;

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
export type Line = { start: Point; end: Point };
export type LineString = { points: Point[] };
export type MultiLineString = { lines: LineString[] };
export type MultiPoint = { points: Point[] };
export type MultiPolygon = { polygons: Polygon[] };
export type Point = { x: number; y: number };
export type Polygon = { exterior: LineString; interior: LineString[] };
