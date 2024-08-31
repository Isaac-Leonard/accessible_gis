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

const border = 16;
const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak(
      "If you are using a screen reader please turn it off to use this application"
    );
    createCanvas();
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

const createCanvas = async () => {
  const canvas = document.createElement("canvas");
  canvas.width = document.documentElement.clientWidth;
  canvas.height = document.documentElement.clientHeight;
  const width = canvas.width - border * 2;
  const height = canvas.height - border * 2;
  const image = await getRawImage();
  const features = await getGeojson();
  console.log("Width: " + width + " and image width: " + image.width);
  console.log("height: " + height + " and image height: " + image.height);
  let scaleFactor = 1;
  let tileX = 0;
  let tileY = 0;
  const gestureManager = new GestureManager(canvas);

  const panLeft = () => {
    if (tileX === 0) {
      speak("At left edge");
      return;
    }
    tileX = tileX - 1;
    speak("panning left");
    render();
    reset();
  };

  const panRight = () => {
    if (tileX >= Math.floor(image.width / (width * scaleFactor))) {
      speak("At right edge");
      return;
    }
    tileX = tileX + 1;
    speak("panning right");
    render();
    reset();
  };

  const panUp = () => {
    if (tileY === 0) {
      speak("At top edge");
      return;
    }
    tileY = tileY - 1;
    speak("Panning up");
    render();
    reset();
  };

  const panDown = () => {
    if (tileY >= Math.floor(image.height / (height * scaleFactor))) {
      speak("At bottom edge");
      return;
    }
    tileY = tileY + 1;
    speak("Panning down");
    render();
    reset();
  };

  gestureManager.addSwipeHandler("left", panLeft);
  gestureManager.addSwipeHandler("right", panRight);
  gestureManager.addSwipeHandler("up", panUp);
  gestureManager.addSwipeHandler("down", panDown);

  const zoomOut = () => {
    scaleFactor *= 2;
    tileX = Math.floor(tileX / 2);
    tileY = Math.floor(tileY / 2);
    speak("Zooming out");
    render();
    reset();
  };

  const zoomIn = () => {
    scaleFactor /= 2;
    tileX = Math.floor(tileX * 2);
    tileY = Math.floor(tileY * 2);
    speak("Zooming in");
    render();
    reset();
  };

  gestureManager.addPinchHandler(zoomOut);
  gestureManager.addSpreadHandler(zoomIn);
  const context = canvas.getContext("2d")!;
  const render = () => {
    const x = width * tileX;
    const y = height * tileY;
    const resizedImage = image.clone().resize({ factor: scaleFactor });
    const newImage = resizedImage.crop({
      x,
      y,
      width: Math.min(width, resizedImage.width - x),
      height: Math.min(height, resizedImage.height - y),
    });
    const imageData = new ImageData(
      Uint8ClampedArray.from(newImage.toBuffer()),
      newImage.width,
      newImage.height
    );

    context.putImageData(imageData, border, border);
    context.putImageData(
      new ImageData(width - newImage.width, height),
      border + newImage.width,
      border
    );
    context.putImageData(
      new ImageData(width, height - newImage.height),
      border,
      border + newImage.height
    );
  };
  root?.appendChild(canvas);
  render();
  let removeHandlers = await manager(canvas);
  const reset = async (): Promise<void> => {
    removeHandlers();
  };
  return canvas;
};

type Line = {
  text: string;
  rect: DOMRect;
};

class OcrManager {
  // Quick hack for now
  // Leave all of the ocr code working but just use an empty detections array instead of scan the image if not enabled
  lines: Line[];

  activeLine: Line | null = null;

  settings: boolean;

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

  constructor(image: ImageData, ocrEnabled: boolean) {
    this.lines = ocrEnabled ? getTextFromImage(image) : [];
    this.settings = ocrEnabled;
  }

  setImage(image: ImageData) {}
  setSettings(settings: boolean) {}
}

export class CanvasManager {
  image: ImageData;
  ctx: CanvasRenderingContext2D;
  connection: WsConnection;
  ocr: OcrManager;
  constructor(public canvas: HTMLCanvasElement) {
    this.bindHandlers();
    this.ctx = canvas.getContext("2d")!;
    this.image = this.ctx.getImageData(0, 0, canvas.width, canvas.height);
    this.connection = new WsConnection(this);
    this.ocr = new OcrManager(this.image, false);
  }

  update(message: AppMessage) {
    switch (message.type) {
      case "Raster":
        return;
      case "Vector":
        return;
    }
  }

  manageFeatures(x: number, y: number) {}

  addListeners() {
    this.canvas.addEventListener("touchstart", this.startHandler);
    this.canvas.addEventListener("touchmove", this.moveHandler);
    this.canvas.addEventListener("touchend", this.endHandler);
    this.canvas.addEventListener("touchcancel", this.cancelHandler);
  }

  bindHandlers() {
    this.startHandler = this.startHandler.bind(this);
    this.moveHandler = this.moveHandler.bind(this);
    this.endHandler = this.endHandler.bind(this);
    this.cancelHandler = this.cancelHandler.bind(this);
  }

  startHandler(this: CanvasManager, e: TouchEvent) {
    e.preventDefault();
    playAudio();
    const y = e.touches[0].pageY;
    const x = e.touches[0].pageX;
    const index = (y * this.image.width + x) * 4;
    const pixel = this.image.data.slice(index, index + 3);
    const average = mean(pixel);
    this.ocr.manageText(x, y);
    this.manageFeatures(x, y);
    setAudioFrequency(mapPixel(average));
  }

  moveHandler(this: CanvasManager, e: TouchEvent) {
    e.preventDefault();
    if (e.currentTarget instanceof HTMLCanvasElement) {
      const touch = e.touches[e.touches.length - 1];
      const y = touch.pageY;
      const x = touch.pageX;
      const index = (y * this.image.width + x) * 4;
      const pixel = this.image.data.slice(index, index + 3);
      const average = mean(pixel);
      this.ocr.manageText(x, y);
      setAudioFrequency(mapPixel(average));
    }
  }

  cancelHandler(this: CanvasManager, e: TouchEvent) {
    e.preventDefault();
    pauseAudio();
  }

  endHandler(this: CanvasManager, e: TouchEvent) {
    e.preventDefault();
    pauseAudio();
  }

  removeListeners(this: CanvasManager) {
    this.canvas.removeEventListener("touchstart", this.startHandler);
    this.canvas.removeEventListener("touchmove", this.moveHandler);
    this.canvas.removeEventListener("touchend", this.endHandler);
    this.canvas.removeEventListener("touchcancel", this.cancelHandler);
  }
}

createButton();
