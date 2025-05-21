import { Feature } from "geojson";
import {
  VectorManager,
  featureCollectionParser,
  pauseAudio,
  playAudio,
  setAudioFrequency,
  speak,
  Raster,
  CoordinateManager,
  getCanvas,
  GestureManager,
} from "touch-device";
import { AppMessage, GisMessage, WsConnection } from "./websocket";

const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = async () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak(
      "If you are using a screen reader please turn it off to use this application"
    );
    new GisManager();
    btn.remove();
  };
  btn.textContent = "Start";
  root?.appendChild(btn);
  return btn;
};

const defaultSettings: GisMessage = {
  raster: { minFreq: 220, maxFreq: 880 },
  vector: { preferedKeys: [] },
};

class GisManager {
  // Required variables
  raster: Raster | null = null;

  coordinateManager = new CoordinateManager();
  settings: GisMessage = defaultSettings;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  connection: WsConnection;
  vectorManager: VectorManager;
  // Initial configuration
  constructor() {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    this.vectorManager = new VectorManager(
      [],
      this.settings.vector,
      this.coordinateManager
    );
    this.gestureManager = new GestureManager(this.canvas);
    this.connection = new WsConnection();
    this.connection.addMessageHandler(this.wsMessageHandler.bind(this));

    setAudioFrequency(440);
    this.coordinateManager.focusFullScreen();
    this.canvas.addEventListener("touchstart", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
      console.log(`screen x: ${screenX}, screen y: ${screenY}`);
      const coords = this.coordinateManager.screenToCoords(screenX, screenY);
      console.log(`Lon: ${coords[0]}, lat: ${coords[1]}`);
      this.vectorManager.speakFeatures(coords);
      this.playAudioInRaster(coords);
    });

    this.canvas.addEventListener("touchmove", (e) => {
      e.preventDefault();
      if (e.targetTouches.length > 1) {
        pauseAudio();
        return;
      }
      const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
      console.log(`screen x: ${screenX}, screen y: ${screenY}`);
      const coords = this.coordinateManager.screenToCoords(screenX, screenY);
      this.vectorManager.speakFeatures(coords);
      this.playAudioInRaster(coords);
    });

    this.canvas.addEventListener("touchend", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      pauseAudio();
    });

    this.canvas.addEventListener("touchcancel", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      pauseAudio();
    });

    this.gestureManager.addPinchHandler(() => {
      const zoomed = this.coordinateManager.zoomOut();
      if (zoomed) {
        speak("Zoomed out");
        this.render();
      } else {
        speak("Cannot zoom out, you may need to swipe down or right");
      }
    });

    this.gestureManager.addSpreadHandler(() => {
      this.coordinateManager.zoomIn();
      speak("Zooming in");
      this.render();
    });

    this.gestureManager.addSwipeHandler("down", () => {
      const scrollDistance = this.coordinateManager.scrollDown();
      if (scrollDistance != 0) {
        speak("Swiped down");
        this.render();
      } else {
        speak("Could not scroll down, at top of map");
      }
    });

    this.gestureManager.addSwipeHandler("up", () => {
      const scrollDistance = this.coordinateManager.scrollUp();
      if (scrollDistance != 0) {
        speak("Swiped up");
        this.render();
      } else {
        speak("Could not scroll up, at bottom of map");
      }
    });

    this.gestureManager.addSwipeHandler("right", () => {
      const scrollDistance = this.coordinateManager.scrollRight();
      if (scrollDistance != 0) {
        speak("Swiped right");
        this.render();
      } else {
        speak("Could not scroll right, at left of map");
      }
    });

    this.gestureManager.addSwipeHandler("left", () => {
      const scrollDistance = this.coordinateManager.scrollLeft();
      if (scrollDistance != 0) {
        speak("Swiped left");
        this.render();
      } else {
        speak("Could not scroll left, at right of map");
      }
    });

    this.getVectors();
    this.getRaster();
  }

  // Functions

  wsMessageHandler(msg: AppMessage) {
    if (msg?.type === "Gis") {
      this.settings = msg.data;
      // speak("Updated settings");
    } else if (msg.type === "FocusRaster") {
      if (this.raster) {
        speak("Focusing raster");
        this.coordinateManager.focusScreen(
          this.raster?.topLeft,
          this.raster?.bottomRight()
        );
        this.render();
      } else {
        speak("Tried to focus raster but no raster is loaded");
      }
    }
  }

  async getVectors() {
    try {
      const res = await fetch("get_vector");
      const geojson = await res
        .json()
        .then((x) => featureCollectionParser.parse(x));
      const features = geojson.features.filter(
        (nullableFeature): nullableFeature is Feature =>
          nullableFeature.geometry !== null
      );
      this.vectorManager.setFeatures(features);
    } catch (e) {
      speak(`Something went wrong with fetching vector data: ${e}`);
      console.log(e);
    }
  }

  renderRaster() {
    if (this.raster === null) {
      return;
    }
    if (
      this.raster.topLeft[0] > this.coordinateManager.rightLon ||
      this.raster.topLeft[1] < this.coordinateManager.bottomLat ||
      this.raster.topLeft[0] + this.raster.width * this.raster.xResolution <
        this.coordinateManager.leftLon ||
      this.raster.topLeft[1] + this.raster.height * this.raster.yResolution >
        this.coordinateManager.topLat
    ) {
      // No raster data is visable
      console.log("Raster off screen");
      return;
    }
    console.log("Rendering raster on screen");
    const topLeftScreen = this.coordinateManager.coordsToScreen(
      this.raster.topLeft
    );
    const bottomRightScreen = this.coordinateManager.coordsToScreen(
      this.raster.rasterToCoords(this.raster.width, this.raster.height)
    );
    const width = bottomRightScreen[0] - topLeftScreen[0];
    const scale = width / this.raster.width;
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

  playAudioInRaster(coords: [number, number]) {
    if (this.raster === null) {
      return;
    }
    const [x, y] = this.raster.coordsToRaster(coords);
    console.log(`lon: ${coords[0]}, x:${x}, lat:${coords[1]}, y:${y}`);
    if (x < 0 || x >= this.raster.width || y < 0 || y >= this.raster.height) {
      pauseAudio();
    } else {
      const index = y * this.raster.width + x;
      const value = this.raster.data.data[index];
      const frequency =
        ((value - this.raster.min) / (this.raster.max - this.raster.min)) *
          (this.settings.raster.maxFreq - this.settings.raster.minFreq) +
        this.settings.raster.minFreq;
      playAudio();
      setAudioFrequency(frequency);
    }
  }

  async getRaster() {
    try {
      console.log("Called get raster");
      const dataRes = await fetch("/get_raster");
      console.log("Fetched data");
      const rasterData = await dataRes.arrayBuffer();
      console.log("Got rasterData array buffer");
      const dataView = new DataView(rasterData);
      const metadata = {
        resolution: dataView.getFloat64(0, true),
        width: Number(dataView.getBigUint64(8, true)),
        height: Number(dataView.getBigUint64(16, true)),
        origin: [
          dataView.getFloat64(24, true),
          dataView.getFloat64(32, true),
        ] as [number, number],
      };
      console.log(metadata);
      const data = new Float32Array(rasterData, 40);
      console.log("Parsed data");
      this.raster = new Raster(
        { type: "Float32", data },
        metadata.origin,
        metadata.width,
        metadata.height,
        metadata.resolution
      );

      this.renderRaster();
    } catch (e) {
      speak(`Something went wrong when fetching raster data: ${e}`);
      console.log(e);
    }
  }

  render() {
    this.ctx.fillStyle = "#000000";
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.vectorManager.render();
    this.renderRaster();
  }
}

createButton();
