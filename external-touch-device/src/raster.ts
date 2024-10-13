import type { ImageConstructorOptions } from "image-js";
import * as ImageJs from "image-js";
import Image from "image-js";

type RasterData =
  | { type: "Uint8"; data: Uint8Array }
  | { type: "Uint16"; data: Uint16Array }
  | { type: "Uint32"; data: Uint32Array }
  | { type: "Int8"; data: Int8Array }
  | { type: "Int16"; data: Int16Array }
  | { type: "Int32"; data: Int32Array }
  | { type: "Float32"; data: Float32Array }
  | { type: "Float64"; data: Float64Array };

export class Raster {
  xResolution: number;
  yResolution: number;
  min: number;
  max: number;
  image: Image;
  constructor(
    public data: RasterData,
    public topLeft: [number, number],
    public width: number,
    public height: number,
    public resolution: number
  ) {
    const { min, max } = getMinMax(data.data);
    this.min = min;
    this.max = max;
    this.xResolution = resolution;
    this.yResolution = -resolution;
    this.image = rasterToGrey(data, width, height, min, max);
  }
  coordsToRaster([lon, lat]: [number, number]) {
    return [
      Math.floor((lon - this.topLeft[0]) / this.xResolution),
      Math.floor((lat - this.topLeft[1]) / this.yResolution),
    ];
  }

  rasterToCoords = (x: number, y: number): [number, number] => [
    x * this.xResolution + this.topLeft[0],
    y * this.yResolution + this.topLeft[1],
  ];

  bottomRight(): [number, number] {
    return this.rasterToCoords(this.width, this.height);
  }
}

const getMinMax = (arr: ArrayLike<number>): { min: number; max: number } => {
  let min = arr[0],
    max = arr[0];
  for (let i = 0; i < arr.length; i++) {
    if (arr[i] < min) {
      min = arr[i];
    } else if (arr[i] > max) {
      max = arr[i];
    }
  }
  return { min, max };
};

const rasterToGrey = (
  data: RasterData,
  width: number,
  height: number,
  min: number,
  max: number
): Image => {
  const options: ImageConstructorOptions = {
    width,
    height,
    kind: "GREY" as ImageJs.ImageKind,
    colorModel: "GREY" as ImageJs.ColorModel,
    components: 1,
    bitDepth: 8,
  };
  switch (data.type) {
    case "Uint8":
      return new Image({ ...options, data: data.data });
    case "Int8":
      return new Image({
        ...options,
        data: Uint8Array.from(data.data, (x) => x + 128),
      });
    default:
      const range = max - min;
      const scaledData = Uint8Array.from(data.data, (x) =>
        Math.round(((x - min) / range) * 256)
      );
      return new Image({ ...options, data: scaledData });
  }
};
