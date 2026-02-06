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
import { AudioTable } from "touch-device/src/raster";
import { colourToString } from "touch-device/src/utils";
import { VectorSettings } from "touch-device/src/vector-manager";

const params = new URLSearchParams(location.search);
const vectorUrl = params.get("vector");
const rasterUrl = params.get("raster");
const settingsUrl = params.get("settings");
const audioTableUrl = params.get("audioTable");

const minLon = Number(params.get("min_lon") ?? -180);
const maxLon = Number(params.get("max_lon") ?? 180);
const minLat = Number(params.get("min_lat") ?? -90);
const maxLat = Number(params.get("max_lat") ?? 90);

const root = document.getElementById("image");

const defaultVectorSettings = {
  audio: {
    prefered_label_field: null,
    announce_leaving: true,
    announce_geometry_type: false,
    radius_for_point_announcements: 5,
    distance_for_line_announcements: 5,
  },
  visual: {
    vector_line_colour: { type: "Named", value: "White" },
    point_radius: 5,
    line_width: 2,
    labels: {
      enabled: false,
      text_colour: { type: "Named", value: "Yellow" },
      font: "",
      line_width: 2,
      fill_text: true,
      prefered_label_field: null,
    },
  },
} as const;

class GisManager {
  // Required variables
  raster: RasterManager;

  coordinateManager: CoordinateManager;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  vectorManager: VectorManager;
  settings = {
    background_colour: { type: "Named", value: "Black" },
  } as const;

  // Initial configuration
  constructor() {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    const bounds = {
      leftLon: minLon,
      rightLon: maxLon,
      topLat: maxLat,
      bottomLat: minLat,
    };
    this.coordinateManager = new CoordinateManager(this.canvas, bounds);
    this.vectorManager = new VectorManager(this.coordinateManager, this.ctx);
    this.raster = new RasterManager(this.coordinateManager, this.canvas);
    this.gestureManager = new GestureManager(this.canvas);

    this.setup();

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

  async setup() {
    const audioTable =
      audioTableUrl !== null ? await this.getAudioTable(audioTableUrl) : null;
    let promises = [];
    if (rasterUrl) {
      promises.push(
        this.raster
          .updateImage({ src: rasterUrl, audioTable })
          .then(() => this.render())
      );
    }
    if (vectorUrl) {
      const settings = settingsUrl
        ? await this.getVectorSettings(decodeURIComponent(settingsUrl))
        : defaultVectorSettings;
      promises.push(
        this.getVectorLayer(decodeURIComponent(vectorUrl)).then((features) => {
          this.vectorManager.createLayer(features, {
            name: vectorUrl,
            settings,
          });
          this.render();
        })
      );
    }
    return await Promise.all(promises);
  }

  async getVectorSettings(src: string): Promise<VectorSettings> {
    const res = await fetch(src);
    const json = await res.json();
    // TODO: Parse this properly.
    return json;
  }

  async getAudioTable(src: string): Promise<AudioTable> {
    const res = await fetch(src);
    const json = await res.json();
    // TODO: Parse this properly
    return json;
  }

  async getVectorLayer(name: string): Promise<FeatureCollection> {
    const res = await fetch(name);
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
    new GisManager();
  };
  btn.textContent = "Start";
  root?.appendChild(btn);
}

app();
