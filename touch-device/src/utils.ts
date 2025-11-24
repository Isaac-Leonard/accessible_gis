import { CssColour } from "./types.ts";

export const mean = (arr: ArrayLike<number>): number => {
  let sum = 0;
  for (let i = 0; i < arr.length; i++) {
    sum = sum + arr[i];
  }
  return sum / arr.length;
};

export const colourToString = (colour: CssColour): string => {
  switch (colour.type) {
    case "Named":
      return colour.value;
    case "Raw":
      return colour.value;
    case "Hex":
      return `#${colour.value}`;
    case "Rgb":
      return `rgb(${colour.value[0]} ${colour.value[1]} ${colour.value[2]})`;
  }
};
