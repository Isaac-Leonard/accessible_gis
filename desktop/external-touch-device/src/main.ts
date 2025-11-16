import { FeatureCollection } from "geojson";
import {
  VectorManager,
  geoJsonParsers,
  speak,
  CoordinateManager,
  getCanvas,
  GestureManager,
  RasterManager,
} from "touch-device";
import { AppMessage, GeneralSettings, WsConnection } from "./websocket";
import { colourToString } from "touch-device/src/vector-manager";

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

class GisManager {
  // Required variables
  raster: RasterManager;

  coordinateManager = new CoordinateManager(this.canvas);
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  connection: WsConnection;
  vectorManager: VectorManager;
  settings: GeneralSettings = {
    background_colour: { type: "Named", value: "Black" },
  };

  // Initial configuration
  constructor() {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    this.vectorManager = new VectorManager(this.coordinateManager, this.ctx);
    this.raster = new RasterManager(this.coordinateManager, this.canvas);
    this.gestureManager = new GestureManager(this.canvas);
    this.connection = new WsConnection();
    this.connection.addMessageHandler(this.wsMessageHandler.bind(this));

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
        this.raster.pauseAudio();
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
      this.raster.pauseAudio();
    });

    this.canvas.addEventListener("touchcancel", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      this.raster.pauseAudio();
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
      switch (msg.type) {
        case "FocusBox":
          speak("Focusing bounding box");
          this.coordinateManager.focusScreen(
            [msg.data[0], msg.data[3]],
            [msg.data[2], msg.data[1]]
          );
          this.render();
          break;
        case "FetchRaster":
          await this.raster.updateImage(msg.data);
          this.render();
          break;
        case "FetchVector":
          const features = await this.getVectorLayer(msg.data.name);
          this.vectorManager.createLayer(features, msg.data);
          this.render();
          break;
        case "UpdateVector":
          this.vectorManager.updateSettingsForLayer(msg.data);
          this.render();
          break;
        case "RemoveVector":
          this.vectorManager.removeLayer(msg.data);
          this.render();
          break;
        case "UpdateGeneralSettings":
          this.settings = msg.data;
          this.render();
          break;
      }
    } catch (e) {
      this.connection.sendError(
        `Something went wrong when processing message: ${e}, ${JSON.stringify(
          e
        )}`
      );
    }
  }

  async getVectorLayer(name: string): Promise<FeatureCollection> {
    const res = await fetch(`get_vector/${name}`);
    const geojson = await res
      .json()
      .then((x) => geoJsonParsers.featureCollectionNonNull.parse(x));
    return geojson;
  }

  render() {
    this.ctx.fillStyle = colourToString(this.settings.background_colour);
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.raster.render();
    this.vectorManager.render();
  }
}

createButton();
