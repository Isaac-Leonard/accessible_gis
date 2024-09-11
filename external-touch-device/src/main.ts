import type { ImageConstructorOptions } from "image-js";
import * as ImageJs from "image-js";
import Image from "image-js";
import * as turf from "@turf/turf";
import { Feature, Position } from "geojson";
import { pauseAudio, playAudio, setAudioFrequency } from "./audio";
import { featureCollection } from "./geojson-parser";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";
import { mean } from "./utils";

const root = document.getElementById("image");

const createButton = () => {
  const btn = document.createElement("button");
  btn.onclick = () => {
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    speak(
      "If you are using a screen reader please turn it off to use this application"
    );
    launchGis();
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

type RasterData =
  | { type: "Uint8"; data: Uint8Array }
  | { type: "Uint16"; data: Uint16Array }
  | { type: "Uint32"; data: Uint32Array }
  | { type: "Int8"; data: Int8Array }
  | { type: "Int16"; data: Int16Array }
  | { type: "Int32"; data: Int32Array }
  | { type: "Float32"; data: Float32Array }
  | { type: "Float64"; data: Float64Array };

type Raster = {
  topLeft: Position;
  xResolution: number;
  yResolution: number;
  data: RasterData;
  width: number;
  height: number;
  min: number;
  max: number;
};

const getMinMax = (arr: ArrayLike<number>): { min: number; max: number } => {
  let min = arr[0],
    max = arr[0];
  for (let i = 0; i < arr.length; i++) {
    if (arr[i] < min) {
      min = arr[i];
    } else if (arr[i] > max) {
      max = arr[i];
    }
  }
  return { min, max };
};

const rasterToGrey = (raster: Raster): Image => {
  const data = raster.data;
  const options: ImageConstructorOptions = {
    width: raster.width,
    height: raster.height,
    kind: "GREY" as ImageJs.ImageKind,
    colorModel: "GREY" as ImageJs.ColorModel,
    components: 1,
    bitDepth: 8,
  };
  switch (data.type) {
    case "Uint8":
      return new Image({
        ...options,
        data: data.data,
      });
    case "Int8":
      return new Image({
        ...options,
        data: Uint8Array.from(data.data, (x) => x + 128),
      });
    default:
      const { min, max } = raster;
      const range = max - min;
      const scaledData = Uint8Array.from(data.data, (x) =>
        Math.round(((x - min) / range) * 256)
      );
      return new Image({ ...options, data: scaledData });
  }
};

const launchGis = () => {
  let features: Feature[] = [];
  const canvas = document.createElement("canvas");
  document.body.appendChild(canvas);
  canvas.width = window.innerWidth;
  canvas.height = document.documentElement.clientHeight;
  const ctx = canvas.getContext("2d")!;
  ctx.fillStyle = "#000000";
  ctx.strokeStyle = "#ffffff";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  setAudioFrequency(440);

  let topLat = maxLat,
    leftLon = minLon,
    bottomLat: number,
    rightLon: number;

  if (canvas.width * 2 < canvas.height) {
    rightLon = maxLon;
    bottomLat = topLat - ((rightLon - leftLon) / canvas.width) * canvas.height;
  } else {
    bottomLat = minLat;
    rightLon = minLon + ((maxLat - minLat) / canvas.height) * canvas.width;
  }

  rightLon = minLon + ((maxLat - minLat) / canvas.height) * canvas.width;

  const screenToCoords = (x: number, y: number): [number, number] => [
    (x / canvas.width) * (rightLon - leftLon) + leftLon,
    -(y / canvas.height) * (topLat - bottomLat) + topLat,
  ];

  const coordsToScreen = ([lon, lat]: [number, number]): [number, number] => [
    ((lon - leftLon) * canvas.width) / (rightLon - leftLon),
    -((lat - topLat) * canvas.height) / (topLat - bottomLat),
  ];

  canvas.addEventListener("touchstart", (e) => {
    e.preventDefault();
    if (e.touches.length > 1) {
      return;
    }
    const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
    console.log(`screen x: ${screenX}, screen y: ${screenY}`);
    const coords = screenToCoords(screenX, screenY);
    console.log(`Lon: ${coords[0]}, lat: ${coords[1]}`);
    speakFeatures(coords);
    playAudioInRaster(coords);
  });

  canvas.addEventListener("touchmove", (e) => {
    e.preventDefault();
    if (e.targetTouches.length > 1) {
      pauseAudio();
      return;
    }
    const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
    const coords = screenToCoords(screenX, screenY);
    speakFeatures(coords);
    playAudioInRaster(coords);
  });

  canvas.addEventListener("touchend", (e) => {
    e.preventDefault();
    if (e.touches.length > 1) {
      return;
    }
    pauseAudio();
  });

  canvas.addEventListener("touchcancel", (e) => {
    e.preventDefault();
    if (e.touches.length > 1) {
      return;
    }
    pauseAudio();
  });

  const gestureManager = new GestureManager(canvas);

  gestureManager.addPinchHandler(() => {
    speak("Zooming out");
    const lonRange = rightLon - leftLon;
    const latRange = topLat - bottomLat;
    const maxXScale = (maxLon - leftLon) / lonRange;
    const maxYScale = (topLat - minLat) / latRange;
    let scale = Math.min(maxXScale, maxYScale, 2);
    if (scale <= 1) {
      speak("Cannot zoom out, you may need to swipe down or right");
      return;
    }
    rightLon = leftLon + lonRange * scale;
    bottomLat = topLat - latRange * scale;
    render();
  });

  gestureManager.addSpreadHandler(() => {
    speak("Zooming in");
    const lonRange = rightLon - leftLon;
    rightLon = leftLon + lonRange / 2;
    const latRange = topLat - bottomLat;
    bottomLat = topLat - latRange / 2;
    render();
  });

  gestureManager.addSwipeHandler("down", () => {
    speak("Swiped down");
    const range = topLat - bottomLat;
    const top = Math.min(topLat + range, maxLat);
    const panDistance = top - topLat;
    topLat = top;
    bottomLat += panDistance;
    render();
  });

  gestureManager.addSwipeHandler("up", () => {
    speak("Swiped up");
    const range = topLat - bottomLat;
    console.log("Range: " + range);
    console.log("Bottom lat: " + bottomLat);
    const bottom = Math.max(bottomLat - range, minLat);
    console.log(`Bottom: ${bottom}`);
    const panDistance = bottomLat - bottom;
    bottomLat = bottom;
    topLat -= panDistance;
    render();
  });

  gestureManager.addSwipeHandler("right", () => {
    speak("Swiped right");
    const range = rightLon - leftLon;
    const left = Math.max(leftLon - range, minLon);
    const panDistance = leftLon - left;
    leftLon = left;
    rightLon -= panDistance;
    render();
  });

  gestureManager.addSwipeHandler("left", () => {
    speak("Swiped left");
    const range = rightLon - leftLon;
    const right = Math.min(rightLon + range, maxLon);
    const panDistance = right - rightLon;
    rightLon = right;
    leftLon += panDistance;
    render();
  });

  const drawPoint = (p: Position) => {
    const [x, y] = coordsToScreen(p as [number, number]);
    ctx.fillRect(x, y, 1, 1);
  };

  const drawLine = (line: Position[]) => {
    ctx.beginPath();
    line.forEach((p) => {
      const [x, y] = coordsToScreen(p as [number, number]);
      ctx.lineTo(x, y);
    });
    ctx.closePath();
    ctx.stroke();
  };

  const getVectors = async () => {
    const res = await fetch("get_vector");
    const geojson = await res.json().then((x) => featureCollection.parse(x));
    features = geojson.features;
    renderVectors();
  };

  const renderVectors = () => {
    ctx.fillStyle = "#ffffff";
    ctx.strokeStyle = "#ffffff";
    features.forEach(({ geometry }) => {
      switch (geometry.type) {
        case "Point":
          drawPoint(geometry.coordinates);
          return;
        case "LineString":
          drawLine(geometry.coordinates);
          return;
        case "Polygon":
          geometry.coordinates.forEach(drawLine);
          return;
        case "MultiPoint":
          geometry.coordinates.forEach(drawPoint);
          return;
        case "MultiLineString":
          geometry.coordinates.forEach(drawLine);
          return;
        case "MultiPolygon":
          geometry.coordinates.forEach((poly) => poly.forEach(drawLine));
          return;
      }
    });
  };

  getVectors();

  const radial = (rightLon - leftLon) / 20;

  let previousFeatures: Feature[] = [];

  const speakFeatures = (coords: [number, number]) => {
    let foundFeatures: Feature[] = [];
    const degrees = { units: "degrees" } as const;
    const geodesic = { method: "geodesic" } as const;
    for (let feature of features) {
      const { geometry } = feature;
      switch (geometry.type) {
        case "Point":
          if (turf.distance(coords, geometry.coordinates, degrees) < radial) {
            foundFeatures.push(feature);
          }
          return;
        case "MultiPoint":
          if (
            geometry.coordinates.some(
              (position) => turf.distance(coords, position, degrees) < radial
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
          if (distanceToLine < radial) {
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
                ) < radial
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
      (feature) => !previousFeatures.includes(feature)
    );

    const leftFeatures = previousFeatures.filter(
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
        return `Left ${name}`;
      })
      .join();
    const text = foundText + "\n" + leftText;
    // Speaking empty text while moving affectively makes any speach while moving impossible.
    if (text.length > 1) {
      speak(text);
    }
    previousFeatures = foundFeatures;
  };

  let raster: Raster;
  let image: Image;

  const coordsToRaster = ([lon, lat]: [number, number]) => [
    Math.floor((lon - raster.topLeft[0]) / raster.xResolution),
    Math.floor((lat - raster.topLeft[1]) / raster.yResolution),
  ];

  // @ts-ignore
  const rasterToCoords = (x: number, y: number): [number, number] => [
    x / raster.xResolution + raster.topLeft[0],
    y / raster.yResolution + raster.topLeft[1],
  ];

  const renderRaster = () => {
    if (
      raster.topLeft[0] > rightLon ||
      raster.topLeft[1] < bottomLat ||
      raster.topLeft[0] + raster.width * raster.xResolution < leftLon ||
      raster.topLeft[1] + raster.height * raster.yResolution > topLat
    ) {
      // No raster data is visable
      console.log("Raster off screen");
      return;
    }
    console.log("Rendering raster on screen");
    const scale = (rightLon - leftLon) / canvas.width / raster.xResolution;
    const transformedImage = image.clone().resize({ factor: scale });
    const screenPosition = coordsToScreen(raster.topLeft as [number, number]);
    const imageData = new ImageData(
      transformedImage.getRGBAData({ clamped: true }) as Uint8ClampedArray,
      transformedImage.width,
      transformedImage.height
    );
    ctx.putImageData(imageData, ...screenPosition);
  };

  const playAudioInRaster = (coords: [number, number]) => {
    const [x, y] = coordsToRaster(coords);
    console.log(`lon: ${coords[0]}, x:${x}, lat:${coords[1]}, y:${y}`);
    if (x < 0 || x >= raster.width || y < 0 || y >= raster.height) {
      pauseAudio();
    } else {
      const index = y * raster.width + x;
      const value = raster.data.data[index];
      const frequency =
        ((value - raster.min) / (raster.max - raster.min)) * 660 + 220;
      playAudio();
      setAudioFrequency(frequency);
    }
  };

  const getRaster = async () => {
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
    const { min, max } = getMinMax(data);
    console.log("Got min max " + min + " and " + max);
    console.log("mean: " + mean(data));
    raster = {
      data: { type: "Float32", data },
      width: metadata.width,
      height: metadata.height,
      min,
      max,
      xResolution: metadata.resolution,
      yResolution: -metadata.resolution,
      topLeft: metadata.origin,
    };

    image = rasterToGrey(raster);
    console.log("Turned to grey image");
    renderRaster();
  };
  getRaster();

  const render = () => {
    ctx.fillStyle = "#000000";
    ctx.strokeStyle = "#ffffff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    renderVectors();
    renderRaster();
  };
  return canvas;
};

createButton();
