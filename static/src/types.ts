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
