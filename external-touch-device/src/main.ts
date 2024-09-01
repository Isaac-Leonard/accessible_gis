import "./websocket";
import turf from "@turf/turf";
import ImageJS, { ImageKind } from "image-js";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio";
import { getTextFromImage } from "./render-image";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";
import { mapPixel, mean, rectContains } from "./utils";
import { RenderMethod } from "./types";
import { featureCollection } from "./geojson-parser";
import { AppMessage, WsConnection } from "./websocket";
import { FeatureCollection } from "geojson";

const border = 16;
const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak(
      "If you are using a screen reader please turn it off to use this application"
    );
    new createCanvas();
    btn.remove();
  };
  btn.textContent = "Start";
  root?.appendChild(btn);
  return btn;
};

const getGeojson = async (): Promise<GeoJSON.FeatureCollection> => {
  const res = await fetch("/get_vector");
  const data = await res.json();
  return featureCollection.parse(data);
};

const getRawImage = async (): Promise<ImageJS> => {
  const renderMethodRes = await fetch("/get_info");
  const renderMethod: RenderMethod = await renderMethodRes.json();
  switch (renderMethod) {
    case "GDAL":
      return await getGdalRasterData();
    case "Image":
      return await getImageFile();
  }
};

const getGdalRasterData = async () => {
  const res = await fetch("/get_image");
  const data = await res.arrayBuffer();
  const view = new DataView(data);
  const width = view.getBigInt64(0, true);
  const height = view.getBigInt64(8, true);
  const dataLen = view.byteLength / 8 - 2;
  let imageData = new Array<number>(dataLen);
  for (let i = 0; i < dataLen; i++) {
    imageData[i] = view.getFloat64((i + 2) * 8);
  }
  const safeImageData = imageData.filter((x) => Number.isFinite(x));
  const { min, max } = safeImageData.reduce(
    ({ min, max }, el) => ({
      min: el < min ? el : min,
      max: el > max ? el : max,
    }),
    {
      min: safeImageData[0],
      max: safeImageData[0],
    }
  );
  const range = max - min;
  const factor = 256 / range;
  const byteData = imageData.map((x) => (x - min) * factor);
  return new ImageJS(Number(width), Number(height), byteData, {
    kind: ImageKind.GREY,
  });
};

const getImageFile = async () => {
  const req = await fetch(`/get_file`);
  const backup = req.clone();
  const blob = await req.blob();
  if (blob.size === 0) {
    return (await ImageJS.load(await backup.arrayBuffer())).rgba8();
  } else {
    const canvas = document.createElement("canvas");
    const url = URL.createObjectURL(blob);
    const imageEl = new Image();
    const imageLoaded = new Promise((res, rej) => {
      imageEl.onload = res;
      imageEl.onerror = rej;
    });
    imageEl.src = url;
    await imageLoaded;
    const ctx = canvas.getContext("2d");
    canvas.width = imageEl.width;
    canvas.height = imageEl.height;
    ctx?.drawImage(imageEl, 0, 0);
    return ImageJS.fromCanvas(canvas).rgba8();
  }
};

export class createCanvas {
  ocr: OcrManager;
  canvas: HTMLCanvasElement;

  image: ImageJS | null = null;
  features: FeatureCollection | null = null;
  width: number;
  height: number;
  scaleFactor: number = 1;
  tileX: number = 0;
  tileY: number = 0;

  gestureManager: GestureManager;

  ctx: CanvasRenderingContext2D;

  connection: WsConnection;

  constructor() {
    this.bindHandlers();
    this.canvas = document.createElement("canvas");
    this.canvas.width = document.documentElement.clientWidth;
    this.canvas.height = document.documentElement.clientHeight;
    this.width = this.canvas.width - border * 2;
    this.height = this.canvas.height - border * 2;
    console.log(
      "Width: " + this.width + " and image width: " + this.image?.width
    );
    console.log(
      "height: " + this.height + " and image height: " + this.image?.height
    );
    this.gestureManager = new GestureManager(this.canvas);

    this.ctx = this.canvas.getContext("2d")!;
    root?.appendChild(this.canvas);
    this.render();
    this.ocr = new OcrManager();
    this.connection = new WsConnection(this);
  }

  addListeners() {
    this.gestureManager.addSwipeHandler("left", this.panLeft);
    this.gestureManager.addSwipeHandler("right", this.panRight);
    this.gestureManager.addSwipeHandler("up", this.panUp);
    this.gestureManager.addSwipeHandler("down", this.panDown);

    this.gestureManager.addPinchHandler(this.zoomOut);
    this.gestureManager.addSpreadHandler(this.zoomIn);

    this.canvas.addEventListener("touchstart", (e) => this.startHandler(e));
    this.canvas.addEventListener("touchmove", (e) => this.moveHandler(e));
    this.canvas.addEventListener("touchend", (e) => this.endHandler(e));
    this.canvas.addEventListener("touchcancel", (e) => this.cancelHandler(e));
  }

  getPixel(x: number, y: number): number[] {
    x = (x + this.width * this.tileX) / 2 ** (this.scaleFactor - 1);
    y = (y + this.height * this.tileY) / 2 ** (this.scaleFactor - 1);
    return this.image?.getPixelXY(x, y)!;
  }

  render() {
    // Clear the canvas before writing fresh data to it
    this.ctx.clearRect(border, border, this.width, this.height);
    if (this.image === null) {
      return;
    }
    const x = this.width * this.tileX;
    const y = this.height * this.tileY;
    const resizedImage = this.image
      .clone()
      .resize({ factor: this.scaleFactor });
    const newImage = resizedImage.crop({
      x,
      y,
      width: Math.min(this.width, resizedImage.width - x),
      height: Math.min(this.height, resizedImage.height - y),
    });
    const imageData = new ImageData(
      Uint8ClampedArray.from(newImage.toBuffer()),
      newImage.width,
      newImage.height
    );

    this.ctx.putImageData(imageData, border, border);
  }

  startHandler(this: createCanvas, e: TouchEvent) {
    e.preventDefault();
    if (this.image === null) {
      speak("No raster image on screen");
      return;
    }

    playAudio();
    const y = e.touches[0].pageY;
    const x = e.touches[0].pageX;
    const pixel = this.getPixel(x - border, y - border);
    const average = mean(pixel);
    this.ocr.manageText(x, y);
    this.manageFeatures(x, y);
    setAudioFrequency(mapPixel(average));
  }

  moveHandler(this: createCanvas, e: TouchEvent) {
    e.preventDefault();
    if (e.currentTarget instanceof HTMLCanvasElement) {
      const touch = e.touches[e.touches.length - 1];
      const y = touch.pageY;
      const x = touch.pageX;
      const pixel = this.getPixel(x - border, y - border);
      const average = mean(pixel);
      this.ocr.manageText(x, y);
      setAudioFrequency(mapPixel(average));
    }
  }

  cancelHandler(this: createCanvas, e: TouchEvent) {
    e.preventDefault();
    pauseAudio();
  }

  endHandler(this: createCanvas, e: TouchEvent) {
    e.preventDefault();
    pauseAudio();
  }

  zoomOut() {
    if (this.image === null) {
      return;
    }

    this.scaleFactor *= 2;
    this.tileX = Math.floor(this.tileX / 2);
    this.tileY = Math.floor(this.tileY / 2);
    speak("Zooming out");
    this.render();
  }

  zoomIn() {
    if (this.image === null) {
      return;
    }

    this.scaleFactor /= 2;
    this.tileX = Math.floor(this.tileX * 2);
    this.tileY = Math.floor(this.tileY * 2);
    speak("Zooming in");
    this.render();
  }

  panLeft() {
    if (this.image === null) {
      return;
    }

    if (this.tileX === 0) {
      speak("At left edge");
      return;
    }
    this.tileX = this.tileX - 1;
    speak("panning left");
    this.render();
  }

  panRight(this: createCanvas) {
    if (this.image === null) {
      return;
    }
    if (
      this.tileX >=
      Math.floor(this.image?.width / (this.width * this.scaleFactor))
    ) {
      speak("At right edge");
      return;
    }
    this.tileX = this.tileX + 1;
    speak("panning right");
    this.render();
  }

  panUp() {
    if (this.image === null) {
      return;
    }
    if (this.tileY === 0) {
      speak("At top edge");
      return;
    }
    this.tileY = this.tileY - 1;
    speak("Panning up");
    this.render();
  }

  panDown() {
    if (this.image === null) {
      return;
    }
    if (
      this.tileY >=
      Math.floor(this.image.height / (this.height * this.scaleFactor))
    ) {
      speak("At bottom edge");
      return;
    }
    this.tileY = this.tileY + 1;
    speak("Panning down");
    this.render();
  }

  async initialise() {
    this.image = await getRawImage();
    this.features = await getGeojson();
    this.ocr.setImage(this.image);
  }

  bindHandlers() {
    this.panLeft = this.panLeft.bind(this);
    this.panRight = this.panRight.bind(this);
    this.panUp = this.panUp.bind(this);
    this.panDown = this.panDown.bind(this);
    this.zoomIn = this.zoomIn.bind(this);
    this.zoomOut = this.zoomOut.bind(this);
    this.startHandler = this.startHandler.bind(this);
    this.moveHandler = this.moveHandler.bind(this);
    this.endHandler = this.endHandler.bind(this);
    this.cancelHandler = this.cancelHandler.bind(this);
  }

  update(message: AppMessage) {
    switch (message.type) {
      case "Raster":
        return;
      case "Vector":
        return;
    }
  }

  manageFeatures(_x: number, _y: number) {}
}

type Line = {
  text: string;
  rect: DOMRect;
};

class OcrManager {
  // Quick hack for now
  // Leave all of the ocr code working but just use an empty detections array instead of scan the image if not enabled
  lines: Line[] = [];

  activeLine: Line | null = null;

  settings: boolean = false;

  manageText(this: OcrManager, x: number, y: number) {
    const line = this.lines.find((line) => rectContains(line.rect, x, y));
    if (line === undefined) {
      this.activeLine = null;
      return;
    }
    if (line === this.activeLine) {
      return;
    }
    this.activeLine = line;
    speak(this.activeLine.text);
  }
  setImage(image: ImageJS | null) {
    const imageData = image
      ?.getCanvas()
      ?.getContext("2d")
      ?.getImageData(0, 0, image.width, image.height);

    if (imageData === null || imageData === undefined) {
      this.lines = [];
      return;
    }
    this.lines = getTextFromImage(imageData);
  }
}

createButton();
