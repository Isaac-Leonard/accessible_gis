import ImageJS from "image-js";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio";
import { getTextFromImage } from "./render-image";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";
import { mapPixel, rectContains } from "./utils";

const border = 16;
const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak("This is a test string");
    createCanvas();
    btn.remove();
  };
  btn.textContent = "Clickme";
  root?.appendChild(btn);
  return btn;
};

const getRawImage = async (): Promise<ImageJS> => {
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

const blackPixel = [0, 0, 0, 0];

const createCanvas = async () => {
  const canvas = document.createElement("canvas");
  canvas.width = document.documentElement.clientWidth;
  canvas.height = document.documentElement.clientHeight;
  const width = canvas.width - border * 2;
  const height = canvas.height - border * 2;
  const image = await getRawImage();
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
  const imageData = context.getImageData(0, 0, canvas.width, canvas.height);
  for (let x = border; x < width + border; x++) {
    for (let y = 0; y < border; y++) {
      imageData.data.set(blackPixel, (y * imageData.width + x) * 4);
      imageData.data.set(
        blackPixel,
        ((imageData.height - y - 1) * imageData.width + x) * 4
      );
    }
  }
  for (let x = 0; x < border; x++) {
    for (let y = 0; y < imageData.height; y++) {
      imageData.data.set(blackPixel, (y * imageData.width + x) * 4);
      const rightPixel = (y * imageData.width + (imageData.width - x - 1)) * 4;
      imageData.data.set(blackPixel, rightPixel);
    }
  }
  context.putImageData(imageData, 0, 0);
  const render = () => {
    const imageData = new ImageData(width, height);
    const x = width * tileX;
    const y = height * tileY;
    const resizedImage = image.clone().resize({ factor: scaleFactor });
    const newImage = resizedImage.crop({
      x,
      y,
      width: Math.min(width, resizedImage.width - x),
      height: Math.min(height, resizedImage.height - y),
    });
    for (let x = 0; x < newImage.width; x++) {
      for (let y = 0; y < newImage.height; y++) {
        const pixel = newImage.getPixelXY(x, y);
        imageData.data.set(pixel, (y * width + x) * 4);
      }
      for (let y = newImage.height; y < height; y++) {
        imageData.data.set(blackPixel, (y * width + x) * 4);
      }
    }
    for (let x = newImage.width; x < width; x++) {
      for (let y = 0; y < height; y++) {
        imageData.data.set(blackPixel, (y * width + x) * 4);
      }
    }
    context.putImageData(imageData, border, border);
  };
  root?.appendChild(canvas);
  canvas.width = document.documentElement.clientWidth;
  canvas.height = document.documentElement.clientHeight;
  render();
  let removeHandlers = manager(canvas);
  const reset = () => {
    removeHandlers();
    removeHandlers = manager(canvas);
  };
  return canvas;
};

const cancelHandler = (e: TouchEvent) => {
  e.preventDefault();
  pauseAudio();
};

const endHandler = (e: TouchEvent) => {
  e.preventDefault();
  pauseAudio();
};

const mean = (arr: ArrayLike<number>): number => {
  let sum = 0;
  for (let i = 0; i < arr.length; i++) {
    sum = sum + arr[i];
  }
  return sum / arr.length;
};

const manager = (canvas: HTMLCanvasElement) => {
  const ctx = canvas.getContext("2d")!;
  const image = ctx.getImageData(0, 0, canvas.width, canvas.height);
  const lines = getTextFromImage(image);
  let activeLine: { text: string; rect: DOMRect } | null = null;

  const manageText = (x: number, y: number) => {
    const line = lines.find((line) => rectContains(line.rect, x, y));
    if (line === undefined) {
      activeLine = null;
      return;
    }
    if (line === activeLine) {
      return;
    }
    activeLine = line;
    speak(activeLine.text);
  };

  const startHandler = (e: TouchEvent) => {
    e.preventDefault();
    playAudio();
    const y = e.touches[0].pageY;
    const x = e.touches[0].pageX;
    const index = (y * image.width + x) * 4;
    const pixel = image.data.slice(index, index + 3);
    const average = mean(pixel);
    manageText(x, y);
    setAudioFrequency(mapPixel(average));
  };

  const moveHandler = (e: TouchEvent) => {
    e.preventDefault();
    if (e.currentTarget instanceof HTMLCanvasElement) {
      const touch = e.touches[e.touches.length - 1];
      const y = touch.pageY;
      const x = touch.pageX;
      const index = (y * image.width + x) * 4;
      const pixel = image.data.slice(index, index + 3);
      const average = mean(pixel);
      manageText(x, y);
      setAudioFrequency(mapPixel(average));
    }
  };
  canvas.addEventListener("touchstart", startHandler);
  canvas.addEventListener("touchmove", moveHandler);
  canvas.addEventListener("touchend", endHandler);
  canvas.addEventListener("touchcancel", cancelHandler);
  return () => {
    canvas.removeEventListener("touchstart", startHandler);
    canvas.removeEventListener("touchmove", moveHandler);
    canvas.removeEventListener("touchend", endHandler);
    canvas.removeEventListener("touchcancel", cancelHandler);
  };
};

createButton();

const host = window.location.host;
const wsUrl = `ws://${host}/ws`;

let socket: WebSocket | null = null;

const connect = () => {
  socket = new WebSocket(wsUrl);

  socket.addEventListener("open", () => {
    console.log("Opened web socket");
  });

  socket.addEventListener("error", (e) => {
    console.log("Error in websocket");
    console.log(e);
    connect();
  });

  socket.addEventListener("message", (e) => {
    console.log("Message recieved");
    console.log(e.data);
  });

  socket.addEventListener("close", (e) => {
    console.log("Closed socket");
    console.log(e);
    connect();
  });
};

connect();

document.addEventListener("visibilitychange", () => {
  if (
    document.visibilityState === "visible" &&
    socket?.readyState === WebSocket.CLOSED
  ) {
    connect();
  }
});
