import * as turf from "@turf/turf";
import { Feature, Position } from "geojson";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio";
import { featureCollection } from "./geojson-parser";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";
import { AppMessage, GisMessage, WsConnection } from "./websocket";
import { Raster } from "touch-device";
import { CoordinateManager } from "touch-device";
import { getCanvas } from "touch-device";

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

  radius = 5;
  previousFeatures: Feature[] = [];
  features: Feature[] = [];
  coordinateManager = new CoordinateManager();
  settings: GisMessage = defaultSettings;
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  connection: WsConnection;

  // Initial configuration
  constructor() {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
    console.log(this.ctx);
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
      this.speakFeatures(coords);
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
      this.speakFeatures(coords);
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

  drawPoint(p: Position) {
    const [x, y] = this.coordinateManager.coordsToScreen(p as [number, number]);
    this.ctx.save();
    this.ctx.beginPath();
    this.ctx.fillStyle = "#ffffff";
    this.ctx.arc(x, y, this.radius, 0, 2 * Math.PI);
    this.ctx.closePath();
    this.ctx.fill();
    this.ctx.restore();
  }

  drawLine(line: Position[]) {
    this.ctx.beginPath();
    line.forEach((p) => {
      const [x, y] = this.coordinateManager.coordsToScreen(
        p as [number, number]
      );
      this.ctx.lineTo(x, y);
    });
    this.ctx.closePath();
    this.ctx.stroke();
  }

  async getVectors() {
    try {
      const res = await fetch("get_vector");
      const geojson = await res.json().then((x) => featureCollection.parse(x));
      this.features = geojson.features.filter(
        (nullableFeature): nullableFeature is Feature =>
          nullableFeature.geometry !== null
      );
      this.renderVectors();
    } catch (e) {
      speak(`Something went wrong with fetching vector data: ${e}`);
      console.log(e);
    }
  }

  renderVectors() {
    this.ctx.fillStyle = "#ffffff";
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.lineWidth = 2;
    this.features.forEach(({ geometry }) => {
      switch (geometry.type) {
        case "Point":
          this.drawPoint(geometry.coordinates);
          return;
        case "LineString":
          this.drawLine(geometry.coordinates);
          return;
        case "Polygon":
          geometry.coordinates.forEach((ring) => this.drawLine(ring));
          return;
        case "MultiPoint":
          geometry.coordinates.forEach((p) => this.drawPoint(p));
          return;
        case "MultiLineString":
          geometry.coordinates.forEach((line) => this.drawLine(line));
          return;
        case "MultiPolygon":
          geometry.coordinates.forEach((poly) =>
            poly.forEach((ring) => this.drawLine(ring))
          );
          return;
      }
    });
  }

  speakFeatures(coords: [number, number]) {
    let foundFeatures: Feature[] = [];
    const degrees = { units: "degrees" } as const;
    const geodesic = { method: "geodesic" } as const;
    for (let feature of this.features) {
      const { geometry } = feature;
      switch (geometry.type) {
        case "Point":
          if (
            turf.distance(coords, geometry.coordinates, degrees) < this.radius
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "MultiPoint":
          if (
            geometry.coordinates.some(
              (position) =>
                turf.distance(coords, position, degrees) < this.radius
            )
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "LineString":
          const distanceToLine = turf.pointToLineDistance(
            coords,
            geometry,
            geodesic
          );
          if (distanceToLine < this.radius) {
            foundFeatures.push(feature);
          }
          continue;
        case "MultiLineString":
          if (
            geometry.coordinates.some(
              (line) =>
                turf.pointToLineDistance(
                  coords,
                  turf.lineString(line),
                  geodesic
                ) < this.radius
            )
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "Polygon":
        case "MultiPolygon":
          if (turf.booleanPointInPolygon(coords, geometry)) {
            foundFeatures.push(feature);
          }
      }
    }

    const featuresToSpeak = foundFeatures.filter(
      (feature) => !this.previousFeatures.includes(feature)
    );

    const leftFeatures = this.previousFeatures.filter(
      (feature) => !foundFeatures.includes(feature)
    );

    const foundText = featuresToSpeak
      .map((feature) => {
        const { geometry, properties } = feature;
        const name = this.getPreferedNameForFeature(properties);
        switch (geometry.type) {
          case "Point":
          case "MultiPoint":
          case "LineString":
          case "MultiLineString":
            return `Near ${geometry.type} ${name}`;
          case "Polygon":
          case "MultiPolygon":
            return `In ${geometry.type} ${name}`;
        }
      })
      .join();

    const leftText = leftFeatures
      .map((feature) => {
        const { properties } = feature;
        const name = this.getPreferedNameForFeature(properties);
        return `Leaving ${name}`;
      })
      .join();
    const text = foundText + "\n" + leftText;
    console.log(text);
    // Speaking empty text while moving affectively makes any speach while moving impossible.
    if (text.length > 1) {
      speak(text);
    }
    this.previousFeatures = foundFeatures;
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
    this.renderVectors();
    this.renderRaster();
  }

  getPreferedNameForFeature(properties: { [name: string]: unknown } | null) {
    if (properties === null) {
      return null;
    }
    return Object.entries(properties).reduce((previous, current) => {
      if (this.settings.vector.preferedKeys.includes(previous[0])) {
        return previous;
      } else if (this.settings.vector.preferedKeys.includes(current[0])) {
        return current;
      } else if (typeof previous[1] === "string") {
        return previous;
      } else if (typeof current[1] === "string" || previous[1] === null) {
        return current;
      } else {
        return previous;
      }
    })[1];
  }
}

createButton();
class Vector {
  previousFeatures: Feature[] = [];
  features: Feature[] = [];
}
