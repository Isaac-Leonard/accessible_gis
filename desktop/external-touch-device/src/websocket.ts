import type { BBox } from "geojson";
import { geoJsonParsers } from "touch-device";
import { VectorSettings } from "touch-device";
import {
  AudioTable,
  AudioType,
  RasterMetadata,
  RasterOptions,
} from "touch-device/src/raster";
import { VectorInfo } from "touch-device/src/vector-manager";
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
  | { type: "FocusBox"; data: BBox }
  | { type: "FetchRaster"; data: RasterOptions }
  | { type: "FetchVector"; data: VectorInfo }
  | { type: "UpdateVector"; data: VectorInfo };

export type ImageMessage = { ocr: boolean };

export const AudioTypeParser: ZodType<AudioType> = z.discriminatedUnion(
  "type",
  [
    z.object({
      type: z.literal("Frequency"),
      value: z.number(),
    }),
    z.object({
      type: z.literal("Silence"),
    }),
    z.object({
      type: z.literal("Speak"),
      value: z.string(),
    }),
    z.object({
      type: z.literal("LinearMap"),
    }),
    z.object({
      type: z.literal("EscSound"),
      value: z.number(),
    }),
  ]
);

export const AudioTableParser: ZodType<AudioTable> = z.object({
  entries: z.array(AudioTypeParser),
  other: AudioTypeParser,
});

const RasterMetadataParser: ZodType<RasterMetadata> = z.object({
  resolution: z.number(),
  width: z.number(),
  height: z.number(),
  noDataValue: z.number().nullable(),
  origin: z.tuple([z.number(), z.number()]),
  audioTable: AudioTableParser.nullable(),
});

const RasterOptionsParser: ZodType<RasterOptions> = z.union([
  z.object({ type: z.literal("RawData"), metadata: RasterMetadataParser }),
  z.object({ type: z.literal("Combined"), metadata: RasterMetadataParser }),
  z.object({ type: z.literal("Image"), metadata: RasterMetadataParser }),
]);

const vectorSettingsParser: ZodType<VectorSettings> = z.object({
  preferedLabel: z.string().nullable(),
  useLabels: z.boolean(),
  announceLeaving: z.boolean(),
  announceGeometryType: z.boolean(),
});

const VectorInfoParser: ZodType<VectorInfo> = z.object({
  name: z.string(),
  settings: vectorSettingsParser,
});

const messageParser: ZodType<AppMessage> = z.union([
  z.object({ type: z.literal("FocusBox"), data: geoJsonParsers.bBox }),
  z.object({ type: z.literal("FetchRaster"), data: RasterOptionsParser }),
  z.object({ type: z.literal("FetchVector"), data: VectorInfoParser }),
  z.object({ type: z.literal("UpdateVector"), data: VectorInfoParser }),
]);
