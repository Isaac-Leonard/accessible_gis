import * as turf from "@turf/turf";
import { Feature, Position } from "geojson";
import { CoordinateManager } from "./coordinate-manager.js";
import { getCanvas } from "./canvas-manager.js";
import { speak } from "./speach.js";

export type VectorSettings = { preferedKeys: string[] };

export class VectorManager {
  radius = 5;

  previousFeatures: Feature[] = [];

  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;

  constructor(
    private features: Feature[],
    private settings: VectorSettings,
    private coordinateManager: CoordinateManager
  ) {
    const { canvas, ctx } = getCanvas();
    this.canvas = canvas;
    this.ctx = ctx;
  }

  setFeatures(features: Feature[]) {
    this.features = features;
    this.previousFeatures = [];
    this.render();
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

  render() {
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
          case "GeometryCollection":
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

  getPreferedNameForFeature(properties: { [name: string]: unknown } | null) {
    if (properties === null) {
      return null;
    }
    return Object.entries(properties).reduce((previous, current) => {
      if (this.settings.preferedKeys.includes(previous[0])) {
        return previous;
      } else if (this.settings.preferedKeys.includes(current[0])) {
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
