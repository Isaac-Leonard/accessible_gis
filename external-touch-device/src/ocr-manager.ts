import ImageJS from "image-js";
import { speak } from "./speach";
import { rectContains } from "./utils";
import { getTextFromImage } from "./render-image";

export type Line = {
  text: string;
  rect: DOMRect;
};

export class OcrManager {
  // Quick hack for now
  // Leave all of the ocr code working but just use an empty detections array instead of scan the image if not enabled
  lines: Line[] = [];

  activeLine: Line | null = null;

  settings: boolean = false;

  manageText(this: OcrManager, x: number, y: number) {
    const line = this.lines.find((line) => rectContains(line.rect, x, y));
    if (line === undefined) {
      this.activeLine = null;
      return;
    }
    if (line === this.activeLine) {
      return;
    }
    this.activeLine = line;
    speak(this.activeLine.text);
  }
  setImage(image: ImageJS | null) {
    const imageData = image
      ?.getCanvas()
      ?.getContext("2d")
      ?.getImageData(0, 0, image.width, image.height);

    if (imageData === null || imageData === undefined) {
      this.lines = [];
      return;
    }
    this.lines = getTextFromImage(imageData);
  }
}
