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
  visibleBounds: CoordinateBox;
  constructor(
    private canvas: HTMLCanvasElement,
    protected outterBounds: CoordinateBox = MaximumBounds,
    visibleBounds?: CoordinateBox
  ) {
    this.visibleBounds = visibleBounds ?? { ...this.outterBounds };
  }

  screenToCoords(x: number, y: number): [number, number] {
    return [
      (x / this.canvas.width) *
        (this.visibleBounds.rightLon - this.visibleBounds.leftLon) +
        this.visibleBounds.leftLon,
      -(y / this.canvas.height) *
        (this.visibleBounds.topLat - this.visibleBounds.bottomLat) +
        this.visibleBounds.topLat,
    ];
  }

  coordsToScreen([lon, lat]: [number, number]): [number, number] {
    return [
      ((lon - this.visibleBounds.leftLon) * this.canvas.width) /
        (this.visibleBounds.rightLon - this.visibleBounds.leftLon),
      -((lat - this.visibleBounds.topLat) * this.canvas.height) /
        (this.visibleBounds.topLat - this.visibleBounds.bottomLat),
    ];
  }

  focusScreen(bounds: CoordinateBox) {
    const screenWidth = this.canvas.width;
    const screenHeight = this.canvas.height;
    const lonRange = bounds.rightLon - bounds.leftLon;
    const latRange = bounds.topLat - bounds.bottomLat;
    this.visibleBounds.topLat = bounds.topLat;
    this.visibleBounds.leftLon = bounds.leftLon;
    const lonOverLat = lonRange / latRange;
    const widthOverHeight = screenWidth / screenHeight;
    if (widthOverHeight > lonOverLat) {
      this.visibleBounds.rightLon = bounds.rightLon;
      const lonOverWidth = lonRange / screenWidth;
      const visibleHeight = lonOverWidth * screenHeight;
      this.visibleBounds.bottomLat = bounds.topLat - visibleHeight;
    } else {
      this.visibleBounds.bottomLat = bounds.bottomLat;
      const latOverHeight = latRange / screenHeight;
      const visibleWidth = latOverHeight * screenWidth;
      this.visibleBounds.rightLon = bounds.leftLon + visibleWidth;
    }
  }

  focusFullScreen() {
    this.focusScreen(this.outterBounds);
  }

  zoomOut() {
    const lonRange = this.visibleBounds.rightLon - this.visibleBounds.leftLon;
    const latRange = this.visibleBounds.topLat - this.visibleBounds.bottomLat;
    const maxXScale =
      (this.outterBounds.rightLon - this.visibleBounds.leftLon) / lonRange;
    const maxYScale =
      (this.visibleBounds.topLat - this.outterBounds.bottomLat) / latRange;
    let scale = Math.min(maxXScale, maxYScale, 2);
    if (scale <= 1) {
      return false;
    }
    this.visibleBounds.rightLon = this.visibleBounds.leftLon + lonRange * scale;
    this.visibleBounds.bottomLat = this.visibleBounds.topLat - latRange * scale;
    return true;
  }

  zoomIn() {
    const lonRange = this.visibleBounds.rightLon - this.visibleBounds.leftLon;
    this.visibleBounds.rightLon = this.visibleBounds.leftLon + lonRange / 2;
    const latRange = this.visibleBounds.topLat - this.visibleBounds.bottomLat;
    this.visibleBounds.bottomLat = this.visibleBounds.topLat - latRange / 2;
  }

  scrollDown(): number {
    const range = this.visibleBounds.topLat - this.visibleBounds.bottomLat;
    const top = Math.min(
      this.visibleBounds.topLat + range,
      this.outterBounds.topLat
    );
    const panDistance = top - this.visibleBounds.topLat;
    this.visibleBounds.topLat = top;
    this.visibleBounds.bottomLat += panDistance;
    return panDistance;
  }

  scrollUp(): number {
    const range = this.visibleBounds.topLat - this.visibleBounds.bottomLat;
    console.log("Range: " + range);
    console.log("Bottom lat: " + this.visibleBounds.bottomLat);
    const bottom = Math.max(
      this.visibleBounds.bottomLat - range,
      this.outterBounds.bottomLat
    );
    console.log(`Bottom: ${bottom}`);
    const panDistance = this.visibleBounds.bottomLat - bottom;
    this.visibleBounds.bottomLat = bottom;
    this.visibleBounds.topLat -= panDistance;
    return panDistance;
  }

  scrollRight(): number {
    const range = this.visibleBounds.rightLon - this.visibleBounds.leftLon;
    const left = Math.max(
      this.visibleBounds.leftLon - range,
      this.outterBounds.leftLon
    );
    const panDistance = this.visibleBounds.leftLon - left;
    this.visibleBounds.leftLon = left;
    this.visibleBounds.rightLon -= panDistance;
    return panDistance;
  }

  scrollLeft(): number {
    const range = this.visibleBounds.rightLon - this.visibleBounds.leftLon;
    const right = Math.min(
      this.visibleBounds.rightLon + range,
      this.outterBounds.rightLon
    );
    const panDistance = right - this.visibleBounds.rightLon;
    this.visibleBounds.rightLon = right;
    this.visibleBounds.leftLon += panDistance;
    return panDistance;
  }
}
