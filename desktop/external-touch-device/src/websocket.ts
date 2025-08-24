import type { BBox } from "geojson";
import { geoJsonParsers } from "touch-device";
import { VectorSettings } from "touch-device";
import { RasterAudioSettings, RasterMetadata } from "touch-device/src/raster";
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
    // this.connect();
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
    // this.connect();
  }

  handleReconnects() {
    document.addEventListener("visibilitychange", () => {
      if (
        document.visibilityState === "visible" &&
        this.socket?.readyState === WebSocket.CLOSED
      ) {
        //        this.connect();
      }
    });
  }

  addMessageHandler(handler: MessageHandler) {
    this.messageHandlers.push(handler);
  }
}

export type AppMessage =
  | { type: "Gis"; data: GisMessage }
  | { type: "FocusBox"; data: BBox }
  | { type: "FetchRaster"; data: RasterMetadata }
  | { type: "FetchVector" };

export type ImageMessage = { ocr: boolean };

export type GisMessage = { vector: VectorSettings };

const RasterMetadataParser: ZodType<RasterMetadata> = z.object({
  resolution: z.number(),
  width: z.number(),
  height: z.number(),
  noDataValue: z.number().nullable(),
  origin: z.tuple([z.number(), z.number()]),
});

const RasterAudioSettingsParser: ZodType<RasterAudioSettings> = z.object({
  minFreq: z.number(),
  maxFreq: z.number(),
});

const vectorSettingsParser = z.object({
  preferedKeys: z.string().array(),
  useLabels: z.boolean(),
  announceLeaving: z.boolean(),
  announceGeometryType: z.boolean(),
});

const GisParser: ZodType<GisMessage> = z.object({
  vector: vectorSettingsParser,
  raster: RasterAudioSettingsParser,
});

const messageParser: ZodType<AppMessage> = z.union([
  z.object({ type: z.literal("Gis"), data: GisParser }),
  z.object({
    type: z.literal("FocusBox"),
    data: geoJsonParsers.bBox,
  }),
  z.object({ type: z.literal("FetchRaster"), data: RasterMetadataParser }),
  z.object({ type: z.literal("FetchVector") }),
]);
