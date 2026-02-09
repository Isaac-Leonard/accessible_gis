import * as turf from "@turf/turf";
import type {
  Feature,
  FeatureCollection,
  GeoJsonProperties,
  Polygon,
  Position,
} from "geojson";
import { CoordinateManager } from "./coordinate-manager.js";
import { speak } from "./speach.js";
import { CssColour } from "./types.ts";
import { colourToString } from "./utils.ts";

export type VectorSettings = {
  audio: TouchDeviceAudioVectorOptions;
  visual: TouchDeviceVisualVectorOptions;
};

export type TouchDeviceVisualVectorOptions = {
  vector_line_colour: CssColour;
  point_radius: number;
  line_width: number;
  labels: TouchDeviceLabelOptions;
};

export type TouchDeviceLabelOptions = {
  enabled: boolean;
  text_colour: CssColour;
  font: string;
  line_width: number;
  fill_text: boolean;
  prefered_label_field: string | null;
};

export type TouchDeviceAudioVectorOptions = {
  prefered_label_field: string | null;
  announce_leaving: boolean;
  announce_geometry_type: boolean;
  radius_for_point_announcements: number;
  distance_for_line_announcements: number;
};

class Layer {
  previousFeatures: Feature[] = [];

  constructor(
    public name: string,
    public features: FeatureCollection,
    public settings: VectorSettings,
    private coordinateManager: CoordinateManager,
    private ctx: CanvasRenderingContext2D
  ) {}

  drawPoint(p: Position) {
    const [x, y] = this.coordinateManager.coordsToScreen(p as [number, number]);
    this.ctx.save();
    this.ctx.beginPath();
    this.ctx.fillStyle = colourToString(
      this.settings.visual.vector_line_colour
    );
    this.ctx.arc(x, y, this.settings.visual.point_radius, 0, 2 * Math.PI);
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
    // Draw the outlines of each vector
    this.ctx.fillStyle = colourToString(
      this.settings.visual.vector_line_colour
    );
    this.ctx.strokeStyle = colourToString(
      this.settings.visual.vector_line_colour
    );
    this.ctx.lineWidth = this.settings.visual.line_width;
    this.features.features.forEach(({ geometry }) => {
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

    // Draw labels over the top of the map
    this.ctx.fillStyle = colourToString(
      this.settings.visual.labels.text_colour
    );
    this.ctx.strokeStyle = colourToString(
      this.settings.visual.labels.text_colour
    );
    this.ctx.lineWidth = this.settings.visual.labels.line_width;
    this.features.features.forEach(({ geometry, properties }) => {
      switch (geometry.type) {
        case "Polygon":
          this.labelPolygon({ type: "Feature", properties, geometry });
          return;
        case "MultiPolygon":
          // Only put the label in the largest polygon to not clutter the map
          let largestArea = 0;
          let largestPolygon;
          for (let poly of geometry.coordinates) {
            let polygon = turf.polygon(poly, properties);
            let area = turf.area(polygon);
            if (area > largestArea) {
              largestArea = area;
              largestPolygon = polygon;
            }
          }
          this.labelPolygon(largestPolygon!);
          return;
        default:
          // We don't label lines or points yet
          return;
      }
    });
  }

  speakFeatures(coords: [number, number]): string {
    let foundFeatures: Feature[] = [];
    const kilometres = { units: "kilometres" } as const;
    const geodesic = { method: "geodesic" } as const;
    for (let feature of this.features.features) {
      const { geometry } = feature;
      switch (geometry.type) {
        case "Point":
          if (
            turf.distance(coords, geometry.coordinates, kilometres) <
            this.settings.audio.radius_for_point_announcements
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "MultiPoint":
          if (
            geometry.coordinates.some(
              (position) =>
                turf.distance(coords, position, kilometres) <
                this.settings.audio.radius_for_point_announcements
            )
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "LineString":
          const distanceToLine = turf.pointToLineDistance(coords, geometry, {
            ...geodesic,
            ...kilometres,
          });
          if (
            distanceToLine < this.settings.audio.distance_for_line_announcements
          ) {
            foundFeatures.push(feature);
          }
          continue;
        case "MultiLineString":
          if (
            geometry.coordinates.some(
              (line) =>
                turf.pointToLineDistance(coords, turf.lineString(line), {
                  ...geodesic,
                  ...kilometres,
                }) < this.settings.audio.distance_for_line_announcements
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

    let text = featuresToSpeak
      .map((feature) => {
        const { geometry, properties } = feature;
        const name = this.getPreferedNameForFeature(properties);
        if (this.settings.audio.announce_geometry_type) {
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
        } else {
          // Convert to string if needed
          return `${name}`;
        }
      })
      .join();

    if (this.settings.audio.announce_leaving) {
      text += "\n";
      text += leftFeatures
        .map((feature) => {
          const { properties } = feature;
          const name = this.getPreferedNameForFeature(properties);
          return `Leaving ${name}`;
        })
        .join();
    }
    this.previousFeatures = foundFeatures;
    console.log(text);
    return text;
  }

  getPreferedNameForFeature(properties: { [name: string]: unknown } | null) {
    if (properties === null) {
      return null;
    }
    const fieldName = this.settings.audio.prefered_label_field;
    return Object.entries(properties).reduce((previous, current) => {
      if (fieldName === previous[0]) {
        return previous;
      } else if (fieldName === current[0]) {
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

  labelPolygon(polygon: Feature<Polygon, GeoJsonProperties>) {
    if (!this.settings.visual.labels.enabled) {
      return;
    }
    const label = `${this.getPreferedNameForFeature(polygon.properties)}`;
    polygon = turf.polygon(
      [polygon.geometry.coordinates[0]],
      polygon.properties
    );

    const labelPoint = this.coordinateManager.coordsToScreen(
      turf.centerOfMass(polygon).geometry.coordinates as [number, number]
    );
    const { width } = this.ctx.measureText(label);
    labelPoint[0] -= width / 2;
    this.ctx.save();
    this.ctx.fillStyle = colourToString(
      this.settings.visual.labels.text_colour
    );
    this.ctx.font = this.settings.visual.labels.font;
    if (this.settings.visual.labels.fill_text) {
      this.ctx.fillText(label, labelPoint[0], labelPoint[1]);
    } else {
      this.ctx.strokeText(label, labelPoint[0], labelPoint[1]);
    }
    this.ctx.restore();
  }
}

export type VectorInfo = { name: string; settings: VectorSettings };

export class VectorManager {
  layers: Layer[] = [];

  constructor(
    private coordinateManager: CoordinateManager,
    private ctx: CanvasRenderingContext2D,
    private preferedVoice?: SpeechSynthesisVoice
  ) {}

  render() {
    this.layers.forEach((layer) => layer.render());
  }

  createLayer(features: FeatureCollection, { name, settings }: VectorInfo) {
    const layer = new Layer(
      name,
      features,
      settings,
      this.coordinateManager,
      this.ctx
    );
    this.layers.push(layer);
  }

  updateSettingsForLayer({ name, settings }: VectorInfo) {
    let layer = this.layers.find((layer) => layer.name === name);
    if (typeof layer !== "undefined") {
      layer.settings = settings;
    }
    this.render();
  }

  removeLayer(name: string) {
    const index = this.layers.findIndex((layer) => layer.name === name);
    if (index !== -1) {
      this.layers.splice(index, 1);
    }
  }

  speakFeatures(coords: [number, number]) {
    const text = this.layers
      .map((layer) => layer.speakFeatures(coords).trim())
      .join("\n")
      .trim();

    // Make sure there's actually text to speak so we don't interupt current speach with nothing
    if (text.length > 0) {
      speak(text, this.preferedVoice);
    }
  }
}
