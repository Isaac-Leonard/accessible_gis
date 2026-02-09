type CoordinateBox = {
  topLat: number;
  leftLon: number;
  bottomLat: number;
  rightLon: number;
};

export const MaximumBounds: CoordinateBox = {
  leftLon: -180,
  bottomLat: -90,
  rightLon: 180,
  topLat: 90,
};

export class CoordinateManager {
  visableBounds: CoordinateBox;
  constructor(
    private canvas: HTMLCanvasElement,
    protected outterBounds: CoordinateBox = MaximumBounds,
    visableBounds?: CoordinateBox
  ) {
    this.visableBounds = visableBounds ?? { ...this.outterBounds };
  }

  screenToCoords(x: number, y: number): [number, number] {
    return [
      (x / this.canvas.width) *
        (this.visableBounds.rightLon - this.visableBounds.leftLon) +
        this.visableBounds.leftLon,
      -(y / this.canvas.height) *
        (this.visableBounds.topLat - this.visableBounds.bottomLat) +
        this.visableBounds.topLat,
    ];
  }

  coordsToScreen([lon, lat]: [number, number]): [number, number] {
    return [
      ((lon - this.visableBounds.leftLon) * this.canvas.width) /
        (this.visableBounds.rightLon - this.visableBounds.leftLon),
      -((lat - this.visableBounds.topLat) * this.canvas.height) /
        (this.visableBounds.topLat - this.visableBounds.bottomLat),
    ];
  }

  focusScreen(bounds: CoordinateBox) {
    const screenWidth = this.canvas.width;
    const screenHeight = this.canvas.height;
    const lonRange = bounds.rightLon - bounds.leftLon;
    const latRange = bounds.topLat - bounds.bottomLat;
    this.visableBounds.topLat = bounds.topLat;
    this.visableBounds.leftLon = bounds.leftLon;
    const lonOverLat = lonRange / latRange;
    const widthOverHeight = screenWidth / screenHeight;
    if (widthOverHeight > lonOverLat) {
      this.visableBounds.rightLon = bounds.rightLon;
      this.visableBounds.bottomLat = bounds.topLat - lonRange / widthOverHeight;
    } else {
      this.visableBounds.bottomLat = bounds.bottomLat;
      this.visableBounds.rightLon = bounds.leftLon + latRange / widthOverHeight;
    }
  }

  focusFullScreen() {
    this.focusScreen(this.outterBounds);
  }

  zoomOut() {
    const lonRange = this.visableBounds.rightLon - this.visableBounds.leftLon;
    const latRange = this.visableBounds.topLat - this.visableBounds.bottomLat;
    const maxXScale =
      (this.outterBounds.rightLon - this.visableBounds.leftLon) / lonRange;
    const maxYScale =
      (this.visableBounds.topLat - this.outterBounds.bottomLat) / latRange;
    let scale = Math.min(maxXScale, maxYScale, 2);
    if (scale <= 1) {
      return false;
    }
    this.visableBounds.rightLon = this.visableBounds.leftLon + lonRange * scale;
    this.visableBounds.bottomLat = this.visableBounds.topLat - latRange * scale;
    return true;
  }

  zoomIn() {
    const lonRange = this.visableBounds.rightLon - this.visableBounds.leftLon;
    this.visableBounds.rightLon = this.visableBounds.leftLon + lonRange / 2;
    const latRange = this.visableBounds.topLat - this.visableBounds.bottomLat;
    this.visableBounds.bottomLat = this.visableBounds.topLat - latRange / 2;
  }

  scrollDown(): number {
    const range = this.visableBounds.topLat - this.visableBounds.bottomLat;
    const top = Math.min(
      this.visableBounds.topLat + range,
      this.outterBounds.topLat
    );
    const panDistance = top - this.visableBounds.topLat;
    this.visableBounds.topLat = top;
    this.visableBounds.bottomLat += panDistance;
    return panDistance;
  }

  scrollUp(): number {
    const range = this.visableBounds.topLat - this.visableBounds.bottomLat;
    console.log("Range: " + range);
    console.log("Bottom lat: " + this.visableBounds.bottomLat);
    const bottom = Math.max(
      this.visableBounds.bottomLat - range,
      this.outterBounds.bottomLat
    );
    console.log(`Bottom: ${bottom}`);
    const panDistance = this.visableBounds.bottomLat - bottom;
    this.visableBounds.bottomLat = bottom;
    this.visableBounds.topLat -= panDistance;
    return panDistance;
  }

  scrollRight(): number {
    const range = this.visableBounds.rightLon - this.visableBounds.leftLon;
    const left = Math.max(
      this.visableBounds.leftLon - range,
      this.outterBounds.leftLon
    );
    const panDistance = this.visableBounds.leftLon - left;
    this.visableBounds.leftLon = left;
    this.visableBounds.rightLon -= panDistance;
    return panDistance;
  }

  scrollLeft(): number {
    const range = this.visableBounds.rightLon - this.visableBounds.leftLon;
    const right = Math.min(
      this.visableBounds.rightLon + range,
      this.outterBounds.rightLon
    );
    const panDistance = right - this.visableBounds.rightLon;
    this.visableBounds.rightLon = right;
    this.visableBounds.leftLon += panDistance;
    return panDistance;
  }
}
