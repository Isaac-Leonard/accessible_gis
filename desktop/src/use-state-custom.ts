import { useEffect, useRef, useState } from "preact/hooks";

export const useCustomState = <T>(state: T) => {
  const [innerState, setState] = useState(state);
  const oldState = useRef(state);

  useEffect(() => {
    oldState.current = deepMerge(oldState.current, innerState) as T;
  }, [innerState]);

  return [oldState.current, setState];
};

export const deepMerge = (target: unknown, src: unknown): unknown => {
  if (target === src) {
    return src;
  } else if (
    typeof target === "string" ||
    typeof target === "number" ||
    typeof target === "undefined" ||
    target === null ||
    typeof target === "function" ||
    typeof src === "bigint" ||
    typeof target === "boolean" ||
    typeof target === "symbol"
  ) {
    return src;
  } else if (
    typeof src === "string" ||
    typeof src === "number" ||
    typeof src === "undefined" ||
    src === null ||
    typeof src === "function" ||
    typeof src === "bigint" ||
    typeof src === "boolean" ||
    typeof src === "symbol"
  ) {
    return src;
  } else if (typeof target !== "object" || typeof src !== "object") {
    throw `Found unexpected non object, target: ${typeof target} and src: ${typeof src}`;
  } else {
    if (Array.isArray(target) && Array.isArray(src)) {
      for (let i = 0; i < src.length; i++) {
        src[i] = deepMerge(target[i], src[i]);
      }
      return src;
    } else if (
      Object.getPrototypeOf(target) !== Object ||
      Object.getPrototypeOf(src) !== Object
    ) {
      throw `Found unexpected prototype of, target: ${Object.getPrototypeOf(
        target
      )} and src: ${Object.getPrototypeOf(src)}`;
    } else {
      for (let key in src) {
        // @ts-ignore
        src[key] = deepMerge(target[key], src[key]);
      }
      return src;
    }
  }
};
