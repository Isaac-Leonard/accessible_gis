import * as turf from "@turf/turf";
import { Feature, Position } from "geojson";
import { pauseAudio, setAudioFrequency } from "./audio";
import { featureCollection } from "./geojson-parser";
import { speak } from "./speach";
import { GestureManager } from "./touch-gpt";

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

const launchGis = () => {
  let features: Feature[] = [];
  const canvas = document.createElement("canvas");
  document.body.appendChild(canvas);
  canvas.width = window.innerWidth;
  canvas.height = document.documentElement.clientHeight;
  const ctx = canvas.getContext("2d")!;
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
    (-y / canvas.height) * (topLat - bottomLat) + topLat,
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
    // playAudio();
    const { screenX, screenY } = e.targetTouches[e.targetTouches.length - 1];
    const coords = screenToCoords(screenX, screenY);
    speakFeatures(coords);
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
    rightLon = Math.min(rightLon - leftLon, maxLon);
    bottomLat = Math.max(bottomLat - (topLat - bottomLat), minLat);
    renderVectors();
  });

  gestureManager.addSpreadHandler(() => {
    speak("Zooming in");
    rightLon = (leftLon + rightLon) / 2;
    topLat = (bottomLat + topLat) / 2;
    renderVectors();
  });

  gestureManager.addSwipeHandler("up", () => {
    speak("Swiped up");
    const range = topLat - bottomLat;
    const top = Math.min(topLat + range, maxLat);
    const panDistance = top - topLat;
    topLat = top;
    bottomLat += panDistance;
    renderVectors();
  });

  gestureManager.addSwipeHandler("down", () => {
    speak("Swiped down");
    const range = topLat - bottomLat;
    const bottom = Math.max(bottomLat - range, minLat);
    const panDistance = bottomLat - bottom;
    bottomLat = bottom;
    topLat -= panDistance;
    renderVectors();
  });

  gestureManager.addSwipeHandler("left", () => {
    speak("Swiped left");
    const range = rightLon - leftLon;
    const left = Math.max(leftLon - range, minLon);
    const panDistance = leftLon - left;
    leftLon = left;
    rightLon -= panDistance;
    renderVectors();
  });

  gestureManager.addSwipeHandler("right", () => {
    speak("Swiped right");
    const range = rightLon - leftLon;
    const right = Math.min(rightLon + range, maxLon);
    const panDistance = right - rightLon;
    rightLon = right;
    leftLon += panDistance;
    renderVectors();
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
  };

  const getVectors = async () => {
    const res = await fetch("get_vector");
    const geojson = await res.json().then((x) => featureCollection.parse(x));
    features = geojson.features;
    renderVectors();
  };

  const renderVectors = () => {
    ctx.fillStyle = "#ffffff";
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

  return canvas;
};

createButton();
