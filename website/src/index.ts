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

type Settings = {
  vector: string | null;
  raster: string | null;
  audioTable: string | null;
  displaySettings: string | null;
  voice: SpeechSynthesisVoice;
};

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
  constructor(settings: Settings) {
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
    this.vectorManager = new VectorManager(
      this.coordinateManager,
      this.ctx,
      settings.voice
    );
    this.raster = new RasterManager(
      this.coordinateManager,
      this.canvas,
      settings.voice
    );
    this.gestureManager = new GestureManager(this.canvas);

    this.setup(settings);

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
        speak("Zoomed out", settings.voice);
        this.render();
      } else {
        speak(
          "Cannot zoom out, you may need to swipe down or right",
          settings.voice
        );
      }
    });

    this.gestureManager.addSpreadHandler(() => {
      this.coordinateManager.zoomIn();
      speak("Zooming in", settings.voice);
      this.render();
    });

    this.gestureManager.addSwipeHandler("down", () => {
      const scrollDistance = this.coordinateManager.scrollDown();
      if (scrollDistance != 0) {
        speak("Swiped down", settings.voice);
        this.render();
      } else {
        speak("Could not scroll down, at top of map", settings.voice);
      }
    });

    this.gestureManager.addSwipeHandler("up", () => {
      const scrollDistance = this.coordinateManager.scrollUp();
      if (scrollDistance != 0) {
        speak("Swiped up", settings.voice);
        this.render();
      } else {
        speak("Could not scroll up, at bottom of map", settings.voice);
      }
    });

    this.gestureManager.addSwipeHandler("right", () => {
      const scrollDistance = this.coordinateManager.scrollRight();
      if (scrollDistance != 0) {
        speak("Swiped right", settings.voice);
        this.render();
      } else {
        speak("Could not scroll right, at left of map", settings.voice);
      }
    });

    this.gestureManager.addSwipeHandler("left", () => {
      const scrollDistance = this.coordinateManager.scrollLeft();
      if (scrollDistance != 0) {
        speak("Swiped left", settings.voice);
        this.render();
      } else {
        speak("Could not scroll left, at right of map", settings.voice);
      }
    });
  }

  // Functions

  async setup(settings: Settings) {
    const audioTable =
      settings.audioTable !== null
        ? await this.getAudioTable(settings.audioTable)
        : null;
    let promises = [];
    if (settings.raster) {
      promises.push(
        this.raster
          .updateImage({ src: settings.raster, audioTable })
          .then(() => this.render())
      );
    }
    if (settings.vector) {
      const displaySettings = settings.displaySettings
        ? await this.getVectorSettings(settings.displaySettings)
        : defaultVectorSettings;
      promises.push(
        this.getVectorLayer(settings.vector).then((features) => {
          this.vectorManager.createLayer(features, {
            name: settings.vector!,
            settings: displaySettings,
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
  const form = document.getElementById("options") as HTMLFormElement;
  const vectorInput = document.getElementById("vector") as HTMLInputElement;
  const rasterInput = document.getElementById("raster") as HTMLInputElement;
  const voiceInput = document.getElementById("voice") as HTMLSelectElement;
  vectorInput.value =
    typeof vectorUrl === "string" ? decodeURIComponent(vectorUrl) : "";
  rasterInput.value =
    typeof rasterUrl === "string" ? decodeURIComponent(rasterUrl) : "";
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    const settings = {
      vector: vectorInput.value.length > 0 ? vectorInput.value : null,
      raster: rasterInput.value.length > 0 ? rasterInput.value : null,
      voice: voices[voiceInput.selectedIndex],
      audioTable: null,
      displaySettings: null,
    };
    root?.replaceChildren();
    speak(
      "If you are using a screen reader please turn it off to use this application",
      settings.voice
    );
    new GisManager(settings);
  });
  const synth = window.speechSynthesis;
  let voices = synth.getVoices();
  const updateVoices = () => {
    voices = synth.getVoices();
    // Clear current selection before repopulating it
    voiceInput.replaceChildren();
    for (let voice of voices) {
      const option = document.createElement("option");
      option.textContent = `${voice.name} (${voice.lang})`;
      if (voice.default) {
        if (voice.lang == window.navigator?.language) {
          option.defaultSelected = true;
        }
        option.textContent += " - Default";
      }
      voiceInput.appendChild(option);
    }
  };
  updateVoices();
  synth.addEventListener("voiceschanged", () => updateVoices());
}

app();
