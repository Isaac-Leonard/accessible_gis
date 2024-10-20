import {
  BBox,
  GeoJsonObject,
  GeoJsonTypes,
  GeometryCollection,
  LineString,
  MultiPolygon,
  Polygon,
  Position,
} from "geojson";
import { Point } from "geojson";
import { MultiLineString } from "geojson";
import { MultiPoint } from "geojson";
import { Geometry } from "geojson";
import { Feature, FeatureCollection } from "geojson";
import * as Z from "zod";

export const geojsonType: Z.ZodType<GeoJsonTypes> = Z.union([
  Z.literal("Point"),
  Z.literal("MultiPoint"),
  Z.literal("LineString"),
  Z.literal("MultiLineString"),
  Z.literal("Polygon"),
  Z.literal("MultiPolygon"),
  Z.literal("GeometryCollection"),
  Z.literal("Feature"),
  Z.literal("FeatureCollection"),
]);

export const bBox: Z.ZodType<BBox> = Z.union([
  Z.tuple([Z.number(), Z.number(), Z.number(), Z.number()]),
  Z.tuple([
    Z.number(),
    Z.number(),
    Z.number(),
    Z.number(),
    Z.number(),
    Z.number(),
  ]),
]);
export const geojsonObject: Z.ZodType<GeoJsonObject> = Z.object({
  type: geojsonType,
  box: bBox.or(Z.undefined()).optional(),
});

// Should be [number, number] | [number, number, number] but seems like only number[] is exported and would rather match the official types for now
const position: Z.ZodType<Position> = Z.array(Z.number());

const point: Z.ZodType<Point> = Z.object({
  type: Z.literal("Point"),
  coordinates: position,
});

const lineString: Z.ZodType<LineString> = Z.object({
  type: Z.literal("LineString"),
  coordinates: Z.array(position),
});

const polygon: Z.ZodType<Polygon> = Z.object({
  type: Z.literal("Polygon"),
  coordinates: Z.array(Z.array(position)),
});

const multiPoint: Z.ZodType<MultiPoint> = Z.object({
  type: Z.literal("MultiPoint"),
  coordinates: Z.array(position),
});

const multiLineString: Z.ZodType<MultiLineString> = Z.object({
  type: Z.literal("MultiLineString"),
  coordinates: Z.array(Z.array(position)),
});

const multiPolygon: Z.ZodType<MultiPolygon> = Z.object({
  type: Z.literal("MultiPolygon"),
  coordinates: Z.array(Z.array(Z.array(position))),
});

const geometryCollection: Z.ZodType<GeometryCollection> = Z.object({
  type: Z.literal("GeometryCollection"),
  geometries: Z.lazy(() => geometry).array(),
});

const geometry: Z.ZodType<Geometry> = Z.union([
  point,
  lineString,
  polygon,
  multiPoint,
  multiLineString,
  multiPolygon,
  geometryCollection,
]);

export const feature: Z.ZodType<Feature<Geometry | null>> = Z.object({
  type: Z.literal("Feature"),
  geometry: geometry.nullable(),
  id: Z.union([Z.string(), Z.number(), Z.undefined()]).optional(),
  properties: Z.record(Z.unknown()),
});

export const featureCollection: Z.ZodType<FeatureCollection<Geometry | null>> =
  Z.object({
    type: Z.literal("FeatureCollection"),
    features: feature.array(),
  });
