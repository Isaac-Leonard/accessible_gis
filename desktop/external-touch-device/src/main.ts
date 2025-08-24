import { Feature } from "geojson";
import {
  VectorManager,
  geoJsonParsers,
  pauseAudio,
  setAudioFrequency,
  speak,
  CoordinateManager,
  getCanvas,
  GestureManager,
  RasterManager,
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
    btn.remove();
    new GisManager();
  };
  btn.textContent = "Start";
  root?.appendChild(btn);
  return btn;
};

const defaultSettings: GisMessage = {
  vector: {
    preferedKeys: [],
    useLabels: false,
    announceLeaving: true,
    announceGeometryType: true,
  },
};

class GisManager {
  // Required variables
  raster: RasterManager;

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
    this.raster = new RasterManager(this.coordinateManager);
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
      this.raster.playAudio(coords);
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
      this.raster.playAudio(coords);
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
  }

  // Functions

  async wsMessageHandler(msg: AppMessage) {
    try {
      if (msg?.type === "Gis") {
        this.settings = msg.data;
        this.vectorManager.setSettings(msg.data.vector);
        this.render();
        // speak("Updated settings");
      } else if (msg.type === "FocusBox") {
        speak("Focusing bounding box");
        this.coordinateManager.focusScreen(
          [msg.data[0], msg.data[3]],
          [msg.data[2], msg.data[1]]
        );
        this.render();
      } else if (msg.type === "FetchRaster") {
        await this.raster.updateImage(msg.data);
        this.render();
      } else if (msg.type === "FetchVector") {
        await this.getVectors();
        this.render();
      }
    } catch (e) {
      this.connection.sendError(
        `Something went wrong when processing message: ${e}, ${JSON.stringify(
          e
        )}`
      );
    }
  }

  async getVectors() {
    try {
      const res = await fetch("get_vector");
      const geojson = await res
        .json()
        .then((x) => geoJsonParsers.featureCollection.parse(x));
      const features = geojson.features.filter(
        (nullableFeature): nullableFeature is Feature =>
          nullableFeature.geometry !== null
      );
      this.vectorManager.setFeatures(features);
    } catch (e) {
      speak(`Something went wrong with fetching vector data: ${e}`);
      console.log(e);
      this.connection.sendError(
        `Something went wrong with fetching vector data: ${e}`
      );
    }
  }

  render() {
    this.ctx.fillStyle = "#000000";
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.raster.render();
    this.vectorManager.render();
  }
}

createButton();
