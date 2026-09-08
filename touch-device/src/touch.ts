import { mean } from "./utils.js";

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

  private oneFingerSwipeHandlers = {
    left: [] as (() => void)[],
    right: [] as (() => void)[],
    up: [] as (() => void)[],
    down: [] as (() => void)[],
  };

  private twoFingerSwipeHandlers = {
    left: [] as (() => void)[],
    right: [] as (() => void)[],
    up: [] as (() => void)[],
    down: [] as (() => void)[],
  };
  private pinchHandlers: (() => void)[] = [];
  private spreadHandlers: (() => void)[] = [];
  private tapHandlers: ((target: HTMLHtmlElement) => void)[] = [];
  private lastTapTime = 0;

  private doubleTapHandlers: (() => void)[] = [];

  constructor(private el: HTMLElement) {
    this.gestureHandlers.push(this.handlePinchZoom.bind(this));
    this.gestureHandlers.push(this.detectOneFingerSwipe.bind(this));
    this.gestureHandlers.push(this.detectTwoFingerSwipe.bind(this));
    this.gestureHandlers.push(this.detectTap.bind(this));
    this.tapHandlers.push(this.detectDoubleTap.bind(this));

    el.addEventListener("touchstart", this.startHandler.bind(this));
    el.addEventListener("touchmove", this.moveHandler.bind(this));
    el.addEventListener("touchend", this.endHandler.bind(this));
    el.addEventListener("touchcancel", this.resetTouches.bind(this));
  }

  private startHandler(ev: TouchEvent) {
    ev.preventDefault();
    for (const touch of Array.from(ev.touches)) {
      if (
        this.startTouches.every((t) => t.touch.identifier !== touch.identifier)
      ) {
        this.startTouches.push(new Touches(touch, ev.timeStamp));
      }
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

  addTwoFingerSwipeHandler(
    direction: "left" | "right" | "up" | "down",
    fn: () => void
  ) {
    this.twoFingerSwipeHandlers[direction].push(fn);
  }

  addOneFingerSwipeHandler(
    direction: "left" | "right" | "up" | "down",
    fn: () => void
  ) {
    this.oneFingerSwipeHandlers[direction].push(fn);
  }

  private handlePinchZoom(start: Touches[], _moves: Touches[], end: Touches[]) {
    const startTouches = start.map((x) => x.touch);
    const endTouches = end.map((x) => x.touch);
    if (startTouches.length !== 2 || endTouches.length !== 2) return;

    const initialDistance = this.calculateDistance(startTouches);
    const finalDistance = this.calculateDistance(endTouches);

    if (initialDistance > finalDistance * 2 && initialDistance > 30) {
      this.gestureInProgress = true;
      this.pinchHandlers.forEach((handler) => handler());
    } else if (finalDistance > initialDistance * 2 && finalDistance > 30) {
      this.gestureInProgress = true;
      this.spreadHandlers.forEach((handler) => handler());
    }
  }

  private calculateDistance(touches: Touch[]): number {
    const xDistance = xDistanceBetween(touches[0], touches[1]);
    const yDistance = yDistanceBetween(touches[0], touches[1]);
    return Math.hypot(xDistance, yDistance);
  }

  private detectOneFingerSwipe(
    start: Touches[],
    _move: Touches[],
    end: Touches[]
  ) {
    if (start.length !== 1 || end.length !== 1) return;

    const startX = start[0].touch.pageX;
    const endX = end[0].touch.pageX;
    const startY = start[0].touch.pageY;
    const endY = end[0].touch.pageY;

    const xDiff = startX - endX;
    const yDiff = startY - endY;

    if (Math.abs(xDiff) > Math.abs(yDiff)) {
      if (xDiff > 0) {
        this.gestureInProgress = true;
        this.oneFingerSwipeHandlers.left.forEach((handler) => handler());
      } else {
        this.gestureInProgress = true;
        this.oneFingerSwipeHandlers.right.forEach((handler) => handler());
      }
    } else {
      // Use > here because screen coordinates have the origin at the top left and increase towards the bottom.
      if (yDiff > 0) {
        this.gestureInProgress = true;
        this.oneFingerSwipeHandlers.up.forEach((handler) => handler());
      } else {
        this.gestureInProgress = true;
        this.oneFingerSwipeHandlers.down.forEach((handler) => handler());
      }
    }
  }

  private detectTwoFingerSwipe(
    start: Touches[],
    _move: Touches[],
    end: Touches[]
  ) {
    if (start.length !== 2 || end.length !== 2) return;

    const startX = mean(start.map((t) => t.touch.pageX));
    const endX = mean(end.map((t) => t.touch.pageX));
    const startY = mean(start.map((t) => t.touch.pageY));
    const endY = mean(end.map((t) => t.touch.pageY));

    const xDiff = startX - endX;
    const yDiff = startY - endY;

    if (Math.abs(xDiff) > Math.abs(yDiff)) {
      if (xDiff > 0) {
        this.gestureInProgress = true;
        this.twoFingerSwipeHandlers.left.forEach((handler) => handler());
      } else {
        this.gestureInProgress = true;
        this.twoFingerSwipeHandlers.right.forEach((handler) => handler());
      }
    } else {
      // Use > here because screen coordinates have the origin at the top left and increase towards the bottom.
      if (yDiff > 0) {
        this.gestureInProgress = true;
        this.twoFingerSwipeHandlers.up.forEach((handler) => handler());
      } else {
        this.gestureInProgress = true;
        this.twoFingerSwipeHandlers.down.forEach((handler) => handler());
      }
    }
  }

  private detectTap(start: Touches[], _move: Touches[], end: Touches[]) {
    if (start.length === 1 && end.length === 1) {
      // Only trigger tap events if a tap is shorter then 200 ms and the start and end of the tap event are on the same element
      if (
        end[0].timeStamp - start[0].timeStamp < 200 &&
        start[0].touch.target === end[0].touch.target
      ) {
        // We cast here as the documentation states that `Touch.target` must be an element which is broader then the EventTarget type specified by default
        // TODO Raise an issue on the typescript github or see if future versions fix this issue
        const target = start[0].touch.target as HTMLHtmlElement;
        this.gestureInProgress = true;
        this.tapHandlers.forEach((fn) => fn(target));
        this.lastTapTime = Date.now();
      }
    }
  }

  private detectDoubleTap() {
    if (Date.now() - this.lastTapTime < 400) {
      this.doubleTapHandlers.forEach((fn) => fn());
    }
  }

  addTapHandler(fn: (target: HTMLHtmlElement) => void) {
    this.tapHandlers.push(fn);
  }

  addDoubleTapHandler(fn: () => void) {
    this.doubleTapHandlers.push(fn);
  }
}
