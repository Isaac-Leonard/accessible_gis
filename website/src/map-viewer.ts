import {
  VectorManager,
  speak,
  CoordinateManager,
  getCanvas,
  GestureManager,
  RasterManager,
} from "touch-device";
import { AudioTable } from "touch-device/src/raster";
import { colourToString } from "touch-device/src/utils";
import { VectorSettings } from "touch-device/src/vector-manager";
import { InitialData } from "./types";

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
} satisfies VectorSettings;

export class GisManager {
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
  voice: SpeechSynthesisVoice;
  keyboardCurrentlyPlaying: boolean = false;
  coords: [number, number] = [0, 0];
  stepSize: number = 1;

  lastTouchTime: number = 0;

  // Initial configuration
  constructor(settings: InitialData) {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    this.voice = settings.voice;
    this.coordinateManager = new CoordinateManager(
      this.canvas,
      settings.bounds
    );
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

    if (typeof this.canvas.requestPointerLock !== "undefined") {
      this.canvas.requestPointerLock();
    }

    if (typeof this.canvas.requestFullscreen !== "undefined") {
      this.canvas.requestFullscreen({ navigationUI: "hide" });
    }
    this.canvas.tabIndex = 1;
    this.canvas.focus({ preventScroll: true });

    this.coordinateManager.focusFullScreen();

    this.canvas.addEventListener("touchstart", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      const { clientX, clientY } = e.targetTouches[e.targetTouches.length - 1];
      console.log(`screen x: ${clientX}, screen y: ${clientY}`);
      this.coords = [clientX, clientY];
      this.playAudio();
    });

    this.canvas.addEventListener("touchmove", (e) => {
      e.preventDefault();
      if (e.targetTouches.length > 1) {
        this.stopAudio();
        this.lastTouchTime = Date.now();
        return;
      }
      const { clientX, clientY } = e.targetTouches[e.targetTouches.length - 1];
      console.log(`screen x: ${clientX}, screen y: ${clientY}`);
      this.coords = [clientX, clientY];
      this.playAudio();
    });

    this.canvas.addEventListener("touchend", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      this.stopAudio();
      this.lastTouchTime = Date.now();
    });

    this.canvas.addEventListener("touchcancel", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      this.stopAudio();
      this.lastTouchTime = Date.now();
    });

    this.gestureManager.addPinchHandler(() => {
      this.zoomOut();
    });

    this.gestureManager.addSpreadHandler(() => {
      if (Date.now() - this.lastTouchTime < 500) {
        this.zoomIn(this.coords);
      } else {
        this.zoomIn();
      }
      this.zoomIn();
    });

    this.gestureManager.addSwipeHandler("down", () => {
      this.scrollDown();
    });

    this.gestureManager.addSwipeHandler("up", () => {
      this.scrollUp();
    });

    this.gestureManager.addSwipeHandler("right", () => {
      this.scrollRight();
    });

    this.gestureManager.addSwipeHandler("left", () => {
      this.scrollLeft();
    });

    this.gestureManager.addDoubleTapHandler(() => {
      const [lon, lat] = this.coordinateManager.screenToCoords(...this.coords);
      speak(`Latitude ${lat.toFixed(3)}, Longitude ${lon.toFixed(3)}`);
    });

    this.canvas.addEventListener("keyup", (e) => {
      this.globalKeyHandler(e);
    });
  }

  // Functions

  playAudio() {
    const coords = this.coordinateManager.screenToCoords(...this.coords);
    this.vectorManager.speakFeatures(coords);
    this.raster.playAudio(coords);
  }

  stopAudio() {
    this.raster.pauseAudio();
  }

  zoomIn(coords?: [number, number]) {
    if (coords) {
      this.coordinateManager.zoomIn(
        this.coordinateManager.screenToCoords(...coords)
      );
      speak("Zooming in on last touch point", this.voice);
    } else {
      this.coordinateManager.zoomIn();
      speak("Zooming in, top left corner held constant", this.voice);
    }
    this.render();
  }

  zoomOut() {
    const zoomed = this.coordinateManager.zoomOut();
    if (zoomed) {
      speak("Zoomed out", this.voice);
      this.render();
    } else {
      speak("Cannot zoom out, you may need to swipe down or right", this.voice);
    }
  }

  scrollDown() {
    const scrollDistance = this.coordinateManager.scrollDown();
    if (scrollDistance != 0) {
      speak("Swiped down", this.voice);
      this.render();
    } else {
      speak("Could not scroll down, at top of map", this.voice);
    }
  }

  scrollUp() {
    const scrollDistance = this.coordinateManager.scrollUp();
    if (scrollDistance != 0) {
      speak("Swiped up", this.voice);
      this.render();
    } else {
      speak("Could not scroll up, at bottom of map", this.voice);
    }
  }

  scrollRight() {
    const scrollDistance = this.coordinateManager.scrollRight();
    if (scrollDistance != 0) {
      speak("Swiped right", this.voice);
      this.render();
    } else {
      speak("Could not scroll right, at left of map", this.voice);
    }
  }

  scrollLeft() {
    const scrollDistance = this.coordinateManager.scrollLeft();
    if (scrollDistance != 0) {
      speak("Swiped left", this.voice);
      this.render();
    } else {
      speak("Could not scroll left, at right of map", this.voice);
    }
  }

  async setup(initialData: InitialData) {
    const audioTable =
      initialData.audioTable !== null
        ? await this.getAudioTable(initialData.audioTable)
        : null;
    let promises = [];
    if (initialData.raster) {
      promises.push(
        this.raster
          .updateImage({ buffer: initialData.raster, audioTable })
          .then(() => this.render())
      );
    }
    for (let vector of initialData.vector) {
      this.vectorManager.createLayer(vector.features, {
        name: vector.name,
        settings: vector.displaySettings ?? defaultVectorSettings,
      });
    }
    this.render();
    try {
      return await Promise.all(promises);
    } catch (e) {
      console.log(e);
    }
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

  render() {
    this.ctx.fillStyle = colourToString(this.settings.background_colour);
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.raster.render();
    this.vectorManager.render();
  }

  globalKeyHandler(e: KeyboardEvent): void {
    if (e.key === "p") {
      e.preventDefault();
      this.keyboardCurrentlyPlaying = !this.keyboardCurrentlyPlaying;
      if (this.keyboardCurrentlyPlaying) {
        this.playAudio();
      } else {
        this.stopAudio();
      }
    }

    if (e.key === "Z") {
      e.preventDefault();
      this.zoomOut();
    }

    if (e.key === "z") {
      e.preventDefault();
      this.zoomIn();
    }

    if (e.key === "c") {
      speak(`Currently at ${this.coords[0]} x, ${this.coords[1]} y`);
    }
    if (e.key === "C") {
      const coords = this.coordinateManager.screenToCoords(...this.coords);
      speak(`Currently at ${coords[0]} east, ${coords[1]} north`);
    }

    if (e.key.startsWith("Arrow") && this.keyboardCurrentlyPlaying) {
      e.preventDefault();
      if (e.ctrlKey) {
        switch (e.key) {
          case "ArrowUp":
            this.scrollUp();
            break;
          case "ArrowDown":
            this.scrollDown();
            break;
          case "ArrowLeft":
            this.scrollLeft();
            break;
          case "ArrowRight":
            this.scrollRight();
            break;
        }
      } else {
        switch (e.key) {
          case "ArrowUp":
            // Moving up but the top left of the screen is [0,0] so we subtract
            this.coords[1] -= this.stepSize;
            break;
          case "ArrowDown":
            // Moving down but the top left of the screen is [0,0] so we add
            this.coords[1] += this.stepSize;
            break;
          case "ArrowLeft":
            this.coords[0] -= this.stepSize;
            break;
          case "ArrowRight":
            this.coords[0] += this.stepSize;
            break;
        }
      }
      this.playAudio();
    }
  }
}
