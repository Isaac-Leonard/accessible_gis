import type { BBox } from "geojson";
import { geoJsonParsers } from "touch-device";
import { VectorSettings } from "touch-device";
import { ZodType, z } from "zod";

const host = window.location.host;
const wsUrl = `ws://${host}/ws`;

export type DeviceMessage =
  | { type: "Voices"; data: string[] }
  | { type: "Error"; data: String };

export type MessageHandler = (message: AppMessage) => void;

export class WsConnection {
  messageHandlers: MessageHandler[] = [];
  // We assign this in the connect method and we call the connect method in the constructor
  socket!: WebSocket;

  constructor() {
    this.connect();
  }

  buffer: string[] = [];

  send(message: DeviceMessage) {
    this.socket.send(JSON.stringify(message));
  }

  sendError(message: string) {
    this.send({ type: "Error", data: message });
  }

  connect() {
    this.socket = new WebSocket(wsUrl);
    this.addListeners();
  }

  addListeners() {
    this.socket.addEventListener("open", (e) => this.onOpen(e));
    this.socket.addEventListener("error", (e) => this.onError(e));
    this.socket.addEventListener("message", (e) => this.onMessage(e));
    this.socket.addEventListener("close", (e) => this.onClose(e));
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
      const message = messageParser.parse(JSON.parse(e.data));
      this.messageHandlers.forEach((handler) => handler(message));
    } catch (e) {
      console.log("An error occured while parsing message from websocket");
      console.log(e);
      this.send({
        type: "Error",
        data: `An error occured while parsing message from websocket: ${JSON.stringify(
          e
        )}`,
      });
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

  addMessageHandler(handler: MessageHandler) {
    this.messageHandlers.push(handler);
  }
}

export type AppMessage =
  | { type: "Image"; data: ImageMessage }
  | { type: "Gis"; data: GisMessage }
  | { type: "FocusBox"; data: BBox }
  | { type: "RefetchRaster" }
  | { type: "RefetchVector" };

export type ImageMessage = { ocr: boolean };

export type GisMessage = { vector: VectorSettings; raster: RasterSettings };

export type AudioSettings = {};

export type RasterSettings = {
  minFreq: number;
  maxFreq: number;
};

const rasterParser: ZodType<RasterSettings> = z.object({
  minFreq: z.number(),
  maxFreq: z.number(),
});

const vectorSettingsParser = z.object({
  preferedKeys: z.string().array(),
  useLabels: z.boolean(),
  announceLeaving: z.boolean(),
  announceGeometryType: z.boolean(),
});

const GisParser = z.object({
  vector: vectorSettingsParser,
  raster: rasterParser,
});

const messageParser: ZodType<AppMessage> = z.union([
  z.object({ type: z.literal("Image"), data: z.object({ ocr: z.boolean() }) }),
  z.object({ type: z.literal("Gis"), data: GisParser }),
  z.object({
    type: z.literal("FocusBox"),
    data: geoJsonParsers.bBox,
  }),
  z.object({ type: z.literal("RefetchRaster") }),
  z.object({ type: z.literal("RefetchVector") }),
]);
