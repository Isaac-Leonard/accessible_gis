import { getCanvas } from "./canvas-manager.js";

export const minLon = -180,
  minLat = -90,
  maxLon = 180,
  maxLat = 90;

export class CoordinateManager {
  topLat = maxLat;
  leftLon = minLon;
  bottomLat: number = minLat;
  rightLon: number = maxLon;
  canvas: HTMLCanvasElement;
  constructor() {
    this.canvas = getCanvas().canvas;
  }

  screenToCoords(x: number, y: number): [number, number] {
    return [
      (x / this.canvas.width) * (this.rightLon - this.leftLon) + this.leftLon,
      -(y / this.canvas.height) * (this.topLat - this.bottomLat) + this.topLat,
    ];
  }

  coordsToScreen([lon, lat]: [number, number]): [number, number] {
    return [
      ((lon - this.leftLon) * this.canvas.width) /
        (this.rightLon - this.leftLon),
      -((lat - this.topLat) * this.canvas.height) /
        (this.topLat - this.bottomLat),
    ];
  }

  focusScreen(
    [minLon, maxLat]: [number, number],
    [maxLon, minLat]: [number, number]
  ) {
    const screenWidth = this.canvas.width;
    const screenHeight = this.canvas.height;
    const lonRange = maxLon - minLon;
    const latRange = maxLat - minLat;
    this.topLat = maxLat;
    this.leftLon = minLon;
    const lonOverLat = lonRange / latRange;
    const widthOverHeight = screenWidth / screenHeight;
    if (widthOverHeight > lonOverLat) {
      this.rightLon = maxLon;
      this.bottomLat = maxLat - (lonRange / screenWidth) * screenHeight;
    } else {
      this.bottomLat = minLat;
      this.rightLon = minLon + (latRange / screenHeight) * screenWidth;
    }
  }

  focusFullScreen() {
    this.focusScreen([minLon, maxLat], [maxLon, minLat]);
  }

  zoomOut() {
    const lonRange = this.rightLon - this.leftLon;
    const latRange = this.topLat - this.bottomLat;
    const maxXScale = (maxLon - this.leftLon) / lonRange;
    const maxYScale = (this.topLat - minLat) / latRange;
    let scale = Math.min(maxXScale, maxYScale, 2);
    if (scale <= 1) {
      return false;
    }
    this.rightLon = this.leftLon + lonRange * scale;
    this.bottomLat = this.topLat - latRange * scale;
    return true;
  }

  zoomIn() {
    const lonRange = this.rightLon - this.leftLon;
    this.rightLon = this.leftLon + lonRange / 2;
    const latRange = this.topLat - this.bottomLat;
    this.bottomLat = this.topLat - latRange / 2;
  }

  scrollDown(): number {
    const range = this.topLat - this.bottomLat;
    const top = Math.min(this.topLat + range, maxLat);
    const panDistance = top - this.topLat;
    this.topLat = top;
    this.bottomLat += panDistance;
    return panDistance;
  }

  scrollUp(): number {
    const range = this.topLat - this.bottomLat;
    console.log("Range: " + range);
    console.log("Bottom lat: " + this.bottomLat);
    const bottom = Math.max(this.bottomLat - range, minLat);
    console.log(`Bottom: ${bottom}`);
    const panDistance = this.bottomLat - bottom;
    this.bottomLat = bottom;
    this.topLat -= panDistance;
    return panDistance;
  }

  scrollRight(): number {
    const range = this.rightLon - this.leftLon;
    const left = Math.max(this.leftLon - range, minLon);
    const panDistance = this.leftLon - left;
    this.leftLon = left;
    this.rightLon -= panDistance;
    return panDistance;
  }

  scrollLeft(): number {
    const range = this.rightLon - this.leftLon;
    const right = Math.min(this.rightLon + range, maxLon);
    const panDistance = right - this.rightLon;
    this.rightLon = right;
    this.leftLon += panDistance;
    return panDistance;
  }
}
