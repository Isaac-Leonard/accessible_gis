import { featureCollection } from "./geojson-parser";
import type { GisViewer } from "./main";
import { ZodType, z } from "zod";

const host = window.location.host;
const wsUrl = `ws://${host}/ws`;

export type DeviceMessage = never;

export class WsConnection {
  // We assign this in the connect method and we call the connect method in the constructor
  socket!: WebSocket;
  manager: GisViewer;

  constructor(manager: GisViewer) {
    this.manager = manager;
    this.bindHandlers();
    this.connect();
  }

  buffer: unknown[] = [];

  send(message: DeviceMessage) {
    this.socket.send(message);
  }

  connect() {
    this.socket = new WebSocket(wsUrl);
    this.addListeners();
  }

  bindHandlers() {
    this.onError = this.onError.bind(this);
    this.onOpen = this.onOpen.bind(this);
    this.onMessage = this.onMessage.bind(this);
    this.onClose = this.onClose.bind(this);
  }

  addListeners() {
    this.socket.addEventListener("open", this.onOpen);
    this.socket.addEventListener("error", this.onError);
    this.socket.addEventListener("message", this.onMessage);
    this.socket.addEventListener("close", this.onClose);
  }

  onOpen(_e: Event) {
    console.log("Opened web socket");
  }

  onError(e: Event) {
    console.log("Error in websocket");
    console.log(e);
    this.connect();
  }

  onMessage(e: MessageEvent) {
    console.log("Message recieved");
    console.log(e.data);
    try {
      const message = messageParser.parse(e.data);
      this.manager.update(message);
    } catch (e) {
      console.log("An error occured while parsing message from websocket");
      console.log(e);
    }
  }

  onClose(e: CloseEvent) {
    console.log("Closed socket");
    console.log(e);
    this.connect();
  }

  handleReconnects() {
    document.addEventListener("visibilitychange", () => {
      if (
        document.visibilityState === "visible" &&
        this.socket?.readyState === WebSocket.CLOSED
      ) {
        this.connect();
      }
    });
  }
}

export type AppMessage =
  | { type: "Raster"; data: RasterrMessage }
  | { type: "Vector"; data: VectorMessage };

type RasterrMessage = RasterSettings;

type VectorMessage = { data: VectorData; settings: VectorSettings };

type VectorData = GeoJSON.FeatureCollection;
type VectorSettings = {};

type AudioSettings = {};

type RasterSettings = {
  enableOcr: boolean;
  image: boolean;
  audio: AudioSettings;
  geoTransform: GeoTransform;
  invertedGeoTransform: GeoTransform;
};

const geoTransformParser = z
  .tuple([
    z.number(),
    z.number(),
    z.number(),
    z.number(),
    z.number(),
    z.number(),
  ])
  .transform((gt) => new GeoTransform(gt));

const rasterParser = z.object({
  enableOcr: z.boolean(),
  image: z.boolean(),
  audio: z.object({}),
  geoTransform: geoTransformParser,
  invertedGeoTransform: geoTransformParser,
});

const vectorSettingsParser = z.object({});

const vectorParser: MyZodType<VectorMessage> = z.object({
  data: featureCollection,
  settings: vectorSettingsParser,
});

const messageParser: MyZodType<AppMessage> = z.union([
  z.object({ type: z.literal("Vector"), data: vectorParser }),
  z.object({ type: z.literal("Raster"), data: rasterParser }),
]);

class GeoTransform {
  constructor(private gt: [number, number, number, number, number, number]) {}

  apply(x: number, y: number) {
    return [
      this.gt[0] + x * this.gt[1] + y * this.gt[2],
      this.gt[3] + x * this.gt[4] + y * this.gt[5],
    ];
  }
}

type MyZodType<T> = ZodType<T, any, any>;
