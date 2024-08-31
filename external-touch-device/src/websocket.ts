import type { CanvasManager } from "./main";
import { ZodType, z } from "zod";

const host = window.location.host;
const wsUrl = `ws://${host}/ws`;

export type DeviceMessage = never;

export class WsConnection {
  socket: WebSocket;
  manager: CanvasManager;
  constructor(manager: CanvasManager) {
    this.manager = manager;
    this.bindHandlers();
    this.connect();
  }

  buffer: unknown[];

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
    connect();
  }
}

document.addEventListener("visibilitychange", () => {
  if (
    document.visibilityState === "visible" &&
    socket?.readyState === WebSocket.CLOSED
  ) {
    connect();
  }
});

type Settings = {};

export type AppMessage = { type: "Raster" } | { type: "Vector" };

type RasterrMessage = { type: "Raster"; data: {} };

type VectorMessage = { type: "Vector"; data: {} };

type UpdateVectorSettings = {};
type UpdateVectorData = {};

type UpdateVectorFull = { data: VectorData; settings: VectorSettings };

type VectorData = GeoJSON.FeatureCollection;
type VectorSettings = {};
type RasterSettings = {
  enableOcr: boolean;
  image: boolean;
  audio: AudioSettings;
};

const rasterParser = z.object({ type: z.literal("Raster") });
const vectorParser = z.union([z.object({ type: z.literal("Settings") })]);

const messageParser: ZodType<AppMessage> = z.union([
  z.object({ type: z.literal("Vector"), data: vectorParser }),
  z.object({ type: z.literal("Raster") }),
]);
