/**
 * Type definition for a gesture handler function.
 * It receives the start, move, and end touches and performs an action.
 */
type GestureHandler = (
  start: Touches[],
  moves: Touches[],
  end: Touches[]
) => void;

/**
 * Calculate the horizontal distance between two touch points.
 * @param a - First touch point
 * @param b - Second touch point
 * @returns The absolute horizontal distance between the two touch points.
 */
const xDistanceBetween = (a: Touch, b: Touch): number => {
  return Math.abs(a.pageX - b.pageX);
};

/**
 * Calculate the vertical distance between two touch points.
 * @param a - First touch point
 * @param b - Second touch point
 * @returns The absolute vertical distance between the two touch points.
 */
const yDistanceBetween = (a: Touch, b: Touch): number => {
  return Math.abs(a.pageY - b.pageY);
};

/**
 * Class representing a single touch point and a timestamp.
 */
class Touches {
  constructor(public touch: Touch, public timeStamp: number) {}

  /**
   * Get the touch point.
   * @returns The touch point.
   */
  getTouch() {
    return this.touch;
  }
}

/**
 * Class for setting up touch event handlers and managing gestures.
 */
export class GestureManager {
  private gestureInProgress = false;
  private gestureHandlers: GestureHandler[] = [];
  private startTouches: Touches[] = [];
  private movedTouches: Touches[] = [];
  private endTouches: Touches[] = [];
  private doubleSwipeHandlers = {
    left: [] as (() => void)[],
    right: [] as (() => void)[],
    up: [] as (() => void)[],
    down: [] as (() => void)[],
  };
  private pinchHandlers: (() => void)[] = [];
  private spreadHandlers: (() => void)[] = [];

  constructor(private el: HTMLElement) {
    this.gestureHandlers.push(this.handlePinchZoom.bind(this));
    this.gestureHandlers.push(this.detectSwipe.bind(this));

    el.addEventListener("touchstart", this.startHandler.bind(this));
    el.addEventListener("touchmove", this.moveHandler.bind(this));
    el.addEventListener("touchend", this.endHandler.bind(this));
    el.addEventListener("touchcancel", this.resetTouches.bind(this));
  }

  private startHandler(ev: TouchEvent) {
    ev.preventDefault();
    for (const touch of Array.from(ev.touches)) {
      this.startTouches.push(new Touches(touch, ev.timeStamp));
    }
  }

  private moveHandler(ev: TouchEvent) {
    ev.preventDefault();
    for (const touch of Array.from(ev.touches)) {
      if (
        this.startTouches.some((t) => t.touch.identifier === touch.identifier)
      ) {
        this.movedTouches.push(new Touches(touch, ev.timeStamp));
      }
    }
  }

  private endHandler(ev: TouchEvent) {
    ev.preventDefault();
    for (const touch of Array.from(ev.changedTouches)) {
      this.endTouches.push(new Touches(touch, ev.timeStamp));
    }

    if (ev.touches.length === 0) {
      this.gestureHandlers.some((handler) => {
        handler(this.startTouches, this.movedTouches, this.endTouches);
        return this.gestureInProgress;
      });
      this.resetTouches();
    }
  }

  private resetTouches() {
    this.startTouches = [];
    this.movedTouches = [];
    this.endTouches = [];
    this.gestureInProgress = false;
  }

  cleanUp() {
    this.el.removeEventListener("touchstart", this.startHandler.bind(this));
    this.el.removeEventListener("touchmove", this.moveHandler.bind(this));
    this.el.removeEventListener("touchend", this.endHandler.bind(this));
  }

  addPinchHandler(fn: () => void) {
    this.pinchHandlers.push(fn);
  }

  addSpreadHandler(fn: () => void) {
    this.spreadHandlers.push(fn);
  }

  addSwipeHandler(direction: "left" | "right" | "up" | "down", fn: () => void) {
    this.doubleSwipeHandlers[direction].push(fn);
  }

  private calcTimeDifference(a: Touches, b: Touches): number {
    return Math.abs(a.timeStamp - b.timeStamp);
  }

  private handlePinchZoom(start: Touches[], _moves: Touches[], end: Touches[]) {
    if (this.calcTimeDifference(start[0], start[start.length - 1]) > 15) {
      return;
    }

    const startTouches = start.map((x) => x.touch);
    const endTouches = end.map((x) => x.touch);
    if (startTouches.length !== 2 || endTouches.length !== 2) return;

    const initialDistance = this.calculateDistance(startTouches);
    const finalDistance = this.calculateDistance(endTouches);

    if (initialDistance > finalDistance * 2) {
      this.gestureInProgress = true;
      this.pinchHandlers.forEach((handler) => handler());
    } else if (finalDistance > initialDistance * 2) {
      this.gestureInProgress = true;
      this.spreadHandlers.forEach((handler) => handler());
    }
  }

  private calculateDistance(touches: Touch[]): number {
    const xDistance = xDistanceBetween(touches[0], touches[1]);
    const yDistance = yDistanceBetween(touches[0], touches[1]);
    return Math.hypot(xDistance, yDistance);
  }

  private detectSwipe(start: Touches[], move: Touches[], _end: Touches[]) {
    if (start.length < 2) return;

    const startX = start[0].touch.pageX;
    const endX = move[move.length - 1].touch.pageX;
    const startY = start[0].touch.pageY;
    const endY = move[move.length - 1].touch.pageY;

    const xDiff = startX - endX;
    const yDiff = startY - endY;

    if (Math.abs(xDiff) > Math.abs(yDiff)) {
      if (xDiff > 100) {
        this.gestureInProgress = true;
        this.doubleSwipeHandlers.left.forEach((handler) => handler());
      } else if (xDiff < -100) {
        this.gestureInProgress = true;
        this.doubleSwipeHandlers.right.forEach((handler) => handler());
      }
    } else {
      if (yDiff > 100) {
        this.gestureInProgress = true;
        this.doubleSwipeHandlers.up.forEach((handler) => handler());
      } else if (yDiff < -100) {
        this.gestureInProgress = true;
        this.doubleSwipeHandlers.down.forEach((handler) => handler());
      }
    }
  }
}
