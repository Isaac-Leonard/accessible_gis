import type { ImageConstructorOptions } from "image-js";
import * as ImageJs from "image-js";
import { Image } from "image-js";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio.ts";
import { getCanvas } from "./canvas-manager.ts";
import { CoordinateManager } from "./coordinate-manager.ts";

export type RasterData =
  | { type: "Uint8"; data: Uint8Array }
  | { type: "Uint16"; data: Uint16Array }
  | { type: "Uint32"; data: Uint32Array }
  | { type: "Int8"; data: Int8Array }
  | { type: "Int16"; data: Int16Array }
  | { type: "Int32"; data: Int32Array }
  | { type: "Float32"; data: Float32Array }
  | { type: "Float64"; data: Float64Array };

export type RasterMetadata = {
  origin: [number, number];
  width: number;
  height: number;
  resolution: number;
  noDataValue: number | null;
};

export type RasterSettings = {
  // While normally calculated on the fly, these can be set by the user to adjust visual and audio contrast
  min: number;
  max: number;
  audio: RasterAudioSettings;
};

export type RasterAudioSettings = { minFreq: number; maxFreq: number };

const getDefaultRasterAudioSettings = (): RasterAudioSettings => ({
  minFreq: 220,
  maxFreq: 880,
});

export type RasterOptions = { metadata: RasterMetadata } & (
  | { type: "RawData" }
  | { type: "Image" }
  | { type: "Combined" }
);
export type Raster = { metadata: RasterMetadata; settings: RasterSettings } & (
  | { type: "RawData"; data: RasterData; image: Image }
  | { type: "Image"; image: Image; data: RasterData }
  | { type: "Combined"; image: Image; data: RasterData }
);

const getDefaultSettings = (
  data: RasterData,
  noDataValue: number | null
): RasterSettings => {
  return {
    ...getMinMax(data.data, noDataValue),
    audio: getDefaultRasterAudioSettings(),
  };
};

export class RasterManager {
  loading: boolean = false;
  raster: Raster | null = null;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;

  constructor(private coordinateManager: CoordinateManager) {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
  }

  async updateImage(options: RasterOptions): Promise<null> {
    switch (options.type) {
      case "RawData":
        return this.getRawDataRaster(options);
      case "Combined":
        return this.getCombinedRaster(options);
      case "Image":
        return this.getImageRaster(options);
    }
  }

  async getRawDataRaster(options: Extract<RasterOptions, { type: "RawData" }>) {
    const data = await this.getRasterData();
    if (data === null) {
      this.raster = null;
      throw new Error("Expected to get raster data and couldn't");
    }
    const settings = getDefaultSettings(data, options.metadata.noDataValue);
    this.raster = {
      type: options.type,
      settings,
      metadata: options.metadata,
      data,
      image: rasterToGrey(
        data,
        options.metadata.width,
        options.metadata.height,
        settings.min,
        settings.max
      ),
    };
    return null;
  }

  async getCombinedRaster(
    options: Extract<RasterOptions, { type: "Combined" }>
  ) {
    const data = await this.getRasterData();
    if (data === null) {
      this.raster = null;
      throw new Error("Expected to get raster data and couldn't");
    }
    const settings = getDefaultSettings(data, options.metadata.noDataValue);
    const image = await Image.load("/get_image");
    this.raster = {
      type: options.type,
      settings,
      metadata: options.metadata,
      data,
      image,
    };
    return null;
  }

  async getImageRaster(options: Extract<RasterOptions, { type: "Image" }>) {
    const image = await Image.load("/get_image");
    const data = this.dataFromImage(image);
    const settings = getDefaultSettings(data, options.metadata.noDataValue);
    this.raster = {
      type: options.type,
      data,
      metadata: options.metadata,
      settings,
      image,
    };
    return null;
  }

  dataFromImage(image: Image): RasterData {
    const data = image.grey().data;
    if (data instanceof Uint8Array) {
      return { type: "Uint8", data };
    } else if (data instanceof Uint16Array) {
      return { type: "Uint16", data };
    } else if (data instanceof Float32Array) {
      return { type: "Float32", data };
    } else {
      throw new Error("Unknown data type for image.");
    }
  }

  async getRasterData(): Promise<RasterData | null> {
    const dataRes = await fetch("/get_raster");
    if (dataRes.status !== 200) {
      return null;
    }
    const rasterData = await dataRes.arrayBuffer();
    const data = new Float64Array(rasterData);
    return { type: "Float64", data };
  }

  coordsToRaster([lon, lat]: [number, number]): [number, number] | null {
    if (this.raster === null) {
      return null;
    }
    return [
      Math.floor(
        (lon - this.raster.metadata.origin[0]) / this.raster.metadata.resolution
      ),
      Math.floor(
        -(lat - this.raster.metadata.origin[1]) /
          this.raster.metadata.resolution
      ),
    ];
  }

  rasterToCoords(x: number, y: number): [number, number] | null {
    if (this.raster === null) {
      return null;
    }
    return [
      x * this.raster.metadata.resolution + this.raster.metadata.origin[0],
      -y * this.raster.metadata.resolution + this.raster.metadata.origin[1],
    ];
  }

  bottomRight(): [number, number] | null {
    if (this.raster === null) {
      return null;
    }
    return this.rasterToCoords(
      this.raster.metadata.width,
      this.raster.metadata.height
    );
  }

  playAudio(coords: [number, number]) {
    if (this.raster === null) {
      return;
    }
    const [x, y] = this.coordsToRaster(coords)!;
    if (
      x < 0 ||
      x >= this.raster.metadata.width ||
      y < 0 ||
      y >= this.raster.metadata.height
    ) {
      pauseAudio();
    } else {
      const index = y * this.raster.metadata.width + x;
      let value = this.raster.data.data[index];
      if (value === this.raster.metadata.noDataValue) {
        value = this.raster.settings.min;
      }
      const frequency =
        ((value - this.raster.settings.min) /
          (this.raster.settings.max - this.raster.settings.min)) *
          (this.raster.settings.audio.maxFreq -
            this.raster.settings.audio.minFreq) +
        this.raster.settings.audio.minFreq;
      playAudio();
      setAudioFrequency(frequency);
    }
  }

  render() {
    if (this.raster === null) {
      return;
    }
    if (
      this.raster.metadata.origin[0] > this.coordinateManager.rightLon ||
      this.raster.metadata.origin[1] < this.coordinateManager.bottomLat ||
      this.raster.metadata.origin[0] +
        this.raster.metadata.width * this.raster.metadata.resolution <
        this.coordinateManager.leftLon ||
      this.raster.metadata.origin[1] +
        this.raster.metadata.height * -this.raster.metadata.resolution >
        this.coordinateManager.topLat
    ) {
      // No raster data is visable
      console.log("Raster off screen");
      return;
    }
    console.log("Rendering raster on screen");
    const topLeftScreen = this.coordinateManager.coordsToScreen(
      this.raster.metadata.origin
    );
    const bottomRightScreen = this.coordinateManager.coordsToScreen(
      this.rasterToCoords(
        this.raster.metadata.width,
        this.raster.metadata.height
      )!
    );
    const width = bottomRightScreen[0] - topLeftScreen[0];
    const scale = width / this.raster.metadata.width;
    const transformedImage = this.raster.image
      .clone()
      .resize({ factor: scale });
    const imageData = new ImageData(
      transformedImage.getRGBAData({ clamped: true }) as Uint8ClampedArray,
      transformedImage.width,
      transformedImage.height
    );
    this.ctx.putImageData(imageData, ...topLeftScreen);
  }
}

const getMinMax = (
  arr: ArrayLike<number>,
  noDataValue: number | null
): { min: number; max: number } => {
  let min = Number.MAX_VALUE,
    max = Number.MIN_VALUE;
  for (let i = 0; i < arr.length; i++) {
    if (arr[i] === noDataValue) {
      continue;
    } else if (arr[i] < min) {
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
