import * as turf from "@turf/turf";
import { Feature, Position } from "geojson";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio";
import { featureCollection } from "./geojson-parser";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";
import { GisMessage, WsConnection } from "./websocket";
import { Raster } from "./raster";

const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = () => {
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

const minLon = -180,
  minLat = -90,
  maxLon = 180,
  maxLat = 90;

const defaultSettings = { raster: { minFreq: 220, maxFreq: 880 }, vector: {} };

class GisManager {
  // Required variables
  raster: Raster | null = null;
  previousFeatures: Feature[] = [];

  radius = 5;

  topLat = maxLat;
  leftLon = minLon;
  bottomLat: number;
  rightLon: number;
  settings: GisMessage = defaultSettings;
  features: Feature[] = [];
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  gestureManager: GestureManager;
  connection: WsConnection;

  // Initial configuration
  constructor() {
    this.canvas = document.createElement("canvas");
    this.ctx = this.canvas.getContext("2d")!;
    console.log(this.ctx);
    this.gestureManager = new GestureManager(this.canvas);
    this.connection = new WsConnection();
    this.connection.addMessageHandler((msg) => {
      if (msg?.type === "Gis") {
        this.settings = msg.data;
        speak("Updated settings");
      }
    });
    document.body.appendChild(this.canvas);
    this.canvas.width = document.documentElement.clientWidth;
    this.canvas.height = document.documentElement.clientHeight;
    this.ctx.fillStyle = "#000000";
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    setAudioFrequency(440);

    if (this.canvas.width * 2 < this.canvas.height) {
      this.rightLon = maxLon;
      this.bottomLat =
        this.topLat -
        ((this.rightLon - this.leftLon) / this.canvas.width) *
          this.canvas.height;
    } else {
      this.bottomLat = minLat;
      this.rightLon =
        minLon + ((maxLat - minLat) / this.canvas.height) * this.canvas.width;
    }

    this.rightLon =
      minLon + ((maxLat - minLat) / this.canvas.height) * this.canvas.width;
    this.canvas.addEventListener("touchstart", (e) => {
      e.preventDefault();
      if (e.touches.length > 1) {
        return;
      }
      const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
      console.log(`screen x: ${screenX}, screen y: ${screenY}`);
      const coords = this.screenToCoords(screenX, screenY);
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
      const coords = this.screenToCoords(screenX, screenY);
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
      speak("Zooming out");
      const lonRange = this.rightLon - this.leftLon;
      const latRange = this.topLat - this.bottomLat;
      const maxXScale = (maxLon - this.leftLon) / lonRange;
      const maxYScale = (this.topLat - minLat) / latRange;
      let scale = Math.min(maxXScale, maxYScale, 2);
      if (scale <= 1) {
        speak("Cannot zoom out, you may need to swipe down or right");
        return;
      }
      this.rightLon = this.leftLon + lonRange * scale;
      this.bottomLat = this.topLat - latRange * scale;
      this.render();
    });

    this.gestureManager.addSpreadHandler(() => {
      speak("Zooming in");
      const lonRange = this.rightLon - this.leftLon;
      this.rightLon = this.leftLon + lonRange / 2;
      const latRange = this.topLat - this.bottomLat;
      this.bottomLat = this.topLat - latRange / 2;
      this.render();
    });

    this.gestureManager.addSwipeHandler("down", () => {
      speak("Swiped down");
      const range = this.topLat - this.bottomLat;
      const top = Math.min(this.topLat + range, maxLat);
      const panDistance = top - this.topLat;
      this.topLat = top;
      this.bottomLat += panDistance;
      this.render();
    });

    this.gestureManager.addSwipeHandler("up", () => {
      speak("Swiped up");
      const range = this.topLat - this.bottomLat;
      console.log("Range: " + range);
      console.log("Bottom lat: " + this.bottomLat);
      const bottom = Math.max(this.bottomLat - range, minLat);
      console.log(`Bottom: ${bottom}`);
      const panDistance = this.bottomLat - bottom;
      this.bottomLat = bottom;
      this.topLat -= panDistance;
      this.render();
    });

    this.gestureManager.addSwipeHandler("right", () => {
      speak("Swiped right");
      const range = this.rightLon - this.leftLon;
      const left = Math.max(this.leftLon - range, minLon);
      const panDistance = this.leftLon - left;
      this.leftLon = left;
      this.rightLon -= panDistance;
      this.render();
    });

    this.gestureManager.addSwipeHandler("left", () => {
      speak("Swiped left");
      const range = this.rightLon - this.leftLon;
      const right = Math.min(this.rightLon + range, maxLon);
      const panDistance = right - this.rightLon;
      this.rightLon = right;
      this.leftLon += panDistance;
      this.render();
    });

    this.getVectors();
    this.getRaster();
  }
  // Functions
  screenToCoords(x: number, y: number): [number, number] {
    return [
      (x / this.canvas.width) * (this.rightLon - this.leftLon) + this.leftLon,
      -(y / this.canvas.height) * (this.topLat - this.bottomLat) + this.topLat,
    ];
  }

  coordsToScreen([lon, lat]: [number, number]): [number, number] {
    return [
      ((lon - this.leftLon) * this.canvas.width) /
        (this.rightLon - this.leftLon),
      -((lat - this.topLat) * this.canvas.height) /
        (this.topLat - this.bottomLat),
    ];
  }

  drawPoint(p: Position) {
    const [x, y] = this.coordsToScreen(p as [number, number]);
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
      const [x, y] = this.coordsToScreen(p as [number, number]);
      this.ctx.lineTo(x, y);
    });
    this.ctx.closePath();
    this.ctx.stroke();
  }

  async getVectors() {
    const res = await fetch("get_vector");
    const geojson = await res.json().then((x) => featureCollection.parse(x));
    this.features = geojson.features;
    this.renderVectors();
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
        const name = properties === null ? null : Object.values(properties)[0];
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
        const name = properties === null ? null : Object.values(properties)[0];
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
      this.raster.topLeft[0] > this.rightLon ||
      this.raster.topLeft[1] < this.bottomLat ||
      this.raster.topLeft[0] + this.raster.width * this.raster.xResolution <
        this.leftLon ||
      this.raster.topLeft[1] + this.raster.height * this.raster.yResolution >
        this.topLat
    ) {
      // No raster data is visable
      console.log("Raster off screen");
      return;
    }
    console.log("Rendering raster on screen");
    const topLeftScreen = this.coordsToScreen(
      this.raster.topLeft as [number, number]
    );
    const bottomRightScreen = this.coordsToScreen(
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
    console.log("Called get raster");
    const metadataRes = await fetch("get_raster_meta");
    console.log("Got metadata");
    const metadata = await metadataRes.json();
    console.log("got metadata json");
    console.log(metadata);
    const dataRes = await fetch("/get_raster");
    console.log("Fetched data");
    const rasterData = await dataRes.arrayBuffer();
    console.log("Got rasterData array buffer");
    const data = new Float32Array(rasterData);
    console.log("Parsed data");
    this.raster = new Raster(
      { type: "Float32", data },
      metadata.origin,
      metadata.width,
      metadata.height,
      metadata.resolution
    );

    this.renderRaster();
  }

  render() {
    this.ctx.fillStyle = "#000000";
    this.ctx.strokeStyle = "#ffffff";
    this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
    this.renderVectors();
    this.renderRaster();
  }
}

createButton();
