import * as ImageJs from "image-js";
import { Image } from "image-js";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio.ts";
import { CoordinateManager } from "./coordinate-manager.ts";
import { speak } from "./speach.ts";
import { fromArrayBuffer, TypedArray } from "geotiff";

export type RasterMetadata = {
  origin: [number, number];
  width: number;
  height: number;
  resolution: number;
  noDataValue: number | null;
};

export type AudioTable = { entries: AudioType[]; other: AudioType };

export type AudioType =
  | { type: "Frequency"; value: number }
  | { type: "Silence" }
  | { type: "Speak"; value: string }
  | { type: "LinearMap" }
  | { type: "EscSound"; value: number };

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

export type Raster = {
  metadata: RasterMetadata;
  settings: RasterSettings;
  image: Image;
  data: TypedArray;
  audioTable: AudioTable | null;
};

export type RasterInfo = { src: string; audioTable: AudioTable | null };

const getDefaultSettings = (
  data: TypedArray,
  noDataValue: number | null
): RasterSettings => {
  return {
    ...getMinMax(data, noDataValue),
    audio: getDefaultRasterAudioSettings(),
  };
};

class SoundManager {
  currentIndex: number | null = null;
  sounds: Map<number, HTMLAudioElement> = new Map();

  constructor(sounds: number[]) {
    for (let index of sounds) {
      const audio = new Audio(`/get_audio/${index}.wav`);
      audio.preload = "auto";
      audio.loop = true;
      this.sounds.set(index, audio);
    }
  }

  async play(index: number) {
    if (this.currentIndex !== index) {
      if (this.currentIndex !== null) {
        this.pause();
      }
      this.sounds.get(index)!.play();
      this.currentIndex = index;
    }
  }

  pause() {
    if (this.currentIndex !== null) {
      this.sounds.get(this.currentIndex)!.pause();
      this.currentIndex = null;
    }
  }
}

export class RasterManager {
  loading: boolean = false;
  raster: Raster | null = null;
  soundManager: SoundManager = new SoundManager([]);

  constructor(
    private coordinateManager: CoordinateManager,
    private canvas: HTMLCanvasElement,
    private preferedVoice?: SpeechSynthesisVoice
  ) {}

  async updateImage(rasterInfo: RasterInfo): Promise<null> {
    const sounds = [...(rasterInfo.audioTable?.entries ?? [])];
    if (rasterInfo.audioTable !== null) {
      sounds.push(rasterInfo.audioTable.other);
    }
    this.soundManager = new SoundManager(
      sounds
        .filter((sound) => sound.type === "EscSound")
        .map((sound) => sound.value)
    );
    return this.getRaster(rasterInfo);
  }

  async processGisRaster(buffer: ArrayBuffer): Promise<{
    metadata: RasterMetadata;
    settings: RasterSettings;
    data: TypedArray;
  }> {
    const tiff = await fromArrayBuffer(buffer);
    const data = await tiff.getImage();
    const metadata: RasterMetadata = {
      width: data.getWidth(),
      height: data.getHeight(),
      origin: data.getOrigin() as [number, number],
      noDataValue: data.getGDALNoData(),
      // TODO: This must be fixed to use the proper resolution and not assume the image uses perfectly square pixels
      resolution: data.getResolution()[0],
    };
    const bands = await data.readRasters({ interleave: false });
    const band = bands[0] as TypedArray;
    const settings = getDefaultSettings(band, metadata.noDataValue);
    return { metadata, data: band, settings };
  }

  async getRaster({ src, audioTable = null }: RasterInfo) {
    const res = await fetch(src);
    const buffer = await res.arrayBuffer();
    const [data, image] = await Promise.all([
      this.processGisRaster(buffer),
      ImageJs.decode(new DataView(buffer)),
    ]);
    this.raster = { ...data, image, audioTable };
    return null;
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
      let value = this.raster.data[index];
      if (this.raster.audioTable !== null) {
        const entry =
          value < this.raster.audioTable.entries.length
            ? this.raster.audioTable.entries[value]
            : this.raster.audioTable.other;
        switch (entry.type) {
          case "Silence":
            this.soundManager.pause();
            pauseAudio();
            return;
          case "Frequency":
            this.soundManager.pause();
            setAudioFrequency(entry.value);
            playAudio();
            return;
          case "Speak":
            this.soundManager.pause();
            pauseAudio();
            speak(entry.value, this.preferedVoice);
            return;
          case "LinearMap":
            this.soundManager.pause();
            const frequency =
              ((value - this.raster.settings.min) /
                (this.raster.settings.max - this.raster.settings.min)) *
                (this.raster.settings.audio.maxFreq -
                  this.raster.settings.audio.minFreq) +
              this.raster.settings.audio.minFreq;
            setAudioFrequency(frequency);
            playAudio();
            return;
          case "EscSound":
            pauseAudio();
            this.soundManager.play(entry.value);
        }
      } else {
        if (value === this.raster.metadata.noDataValue) {
          value = this.raster.settings.min;
        }
        const frequency =
          ((value - this.raster.settings.min) /
            (this.raster.settings.max - this.raster.settings.min)) *
            (this.raster.settings.audio.maxFreq -
              this.raster.settings.audio.minFreq) +
          this.raster.settings.audio.minFreq;
        setAudioFrequency(frequency);
        playAudio();
      }
    }
  }

  render() {
    if (this.raster === null) {
      return;
    }
    if (
      this.raster.metadata.origin[0] >
        this.coordinateManager.visibleBounds.rightLon ||
      this.raster.metadata.origin[1] <
        this.coordinateManager.visibleBounds.bottomLat ||
      this.raster.metadata.origin[0] +
        this.raster.metadata.width * this.raster.metadata.resolution <
        this.coordinateManager.visibleBounds.leftLon ||
      this.raster.metadata.origin[1] +
        this.raster.metadata.height * -this.raster.metadata.resolution >
        this.coordinateManager.visibleBounds.topLat
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
      .resize({ xFactor: scale, yFactor: scale });
    ImageJs.writeCanvas(transformedImage, this.canvas, {
      dx: topLeftScreen[0],
      dy: topLeftScreen[1],
      resizeCanvas: false,
    });
  }

  pauseAudio() {
    pauseAudio();
    this.soundManager.pause();
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
