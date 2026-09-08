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
import { colourToString } from "touch-device/src/utils";

const root = document.getElementById("image");

class GisManager {
  // Required variables
  raster: RasterManager;

  coordinateManager: CoordinateManager;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  vectorManager: VectorManager;
  settings: GeneralSettings = {
    background_colour: { type: "Named", value: "Black" },
  };

  // Initial configuration
  constructor() {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    this.coordinateManager = new CoordinateManager(this.canvas);
    this.vectorManager = new VectorManager(this.coordinateManager, this.ctx);
    this.raster = new RasterManager(this.coordinateManager, this.canvas);
    this.gestureManager = new GestureManager(this.canvas);

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

    this.gestureManager.addTwoFingerSwipeHandler("down", () => {
      const scrollDistance = this.coordinateManager.scrollDown();
      if (scrollDistance != 0) {
        speak("Swiped down");
        this.render();
      } else {
        speak("Could not scroll down, at top of map");
      }
    });

    this.gestureManager.addTwoFingerSwipeHandler("up", () => {
      const scrollDistance = this.coordinateManager.scrollUp();
      if (scrollDistance != 0) {
        speak("Swiped up");
        this.render();
      } else {
        speak("Could not scroll up, at bottom of map");
      }
    });

    this.gestureManager.addTwoFingerSwipeHandler("right", () => {
      const scrollDistance = this.coordinateManager.scrollRight();
      if (scrollDistance != 0) {
        speak("Swiped right");
        this.render();
      } else {
        speak("Could not scroll right, at left of map");
      }
    });

    this.gestureManager.addTwoFingerSwipeHandler("left", () => {
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

  async update(msg: AppMessage) {
    switch (msg.type) {
      case "FocusBox":
        speak("Focusing bounding box");
        this.coordinateManager.focusScreen({
          leftLon: msg.data[0],
          topLat: msg.data[3],
          rightLon: msg.data[2],
          bottomLat: msg.data[1],
        });
        this.render();
        break;
      case "FetchRaster":
        const { src, audioTable } = msg.data;
        const res = await fetch(src);
        const buffer = await res.arrayBuffer();
        await this.raster.updateImage({ buffer, audioTable });
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
    return null;
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

function app() {
  const btn = document.createElement("button");
  btn.onclick = async () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak(
      "If you are using a screen reader please turn it off to use this application"
    );
    btn.remove();
    const gis = new GisManager();
    const connection = new WsConnection();
    connection.addMessageHandler((msg) => {
      try {
        gis.update(msg);
      } catch (e) {
        return e as Error;
      }
    });
  };
  btn.textContent = "Start";
  root?.appendChild(btn);
}

app();
