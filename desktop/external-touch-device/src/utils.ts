import { FetchedImageData } from "./types";

/**
 * Return true if the point `(x, y)` is contained within `r`.
 *
 * The left/top edges are treated as "inside" the rect and the bottom/right
 * edges as "outside". This ensures that for adjacent rects, a point will only
 * lie within one rect.
 */
export function rectContains(r: DOMRect, x: number, y: number) {
  return x >= r.left && x < r.right && y >= r.top && y < r.bottom;
}

export function mapPixel(pixel: number) {
  const minFreq = 220 / 4;
  const maxFreq = 880;
  return (pixel * (maxFreq - minFreq)) / 255 + minFreq;
}

export function grayToImageData(data: number[], width: number, height: number) {
  // An array of grey scale pixels
  if (width * height !== data.length) {
    throw new Error("width*height should equal data.length");
  }
  const imgData = new ImageData(width, height);

  for (let i = 0; i < data.length; i++) {
    imgData.data[i * 4 + 0] = data[i];
    imgData.data[i * 4 + 1] = data[i];
    imgData.data[i * 4 + 2] = data[i];
    imgData.data[i * 4 + 3] = 255;
  }
  return imgData;
}

export function convertTo8Bit(image: FetchedImageData): number[] {
  if (image.type === "UInt8") {
    return image.data;
  }
  const output = new Array<number>(image.data.length);
  if (image.type === "Int8") {
    for (let i = 0; i < image.data.length; i++) {
      output[i] = image.data[i] + 128;
    }
  }
  let min = image.data[0];
  let max = image.data[0];
  for (let pixel of image.data) {
    if (pixel === image.no_data_value) {
      continue;
    }
    if (pixel < min) {
      min = pixel;
    } else if (pixel > max) {
      max = pixel;
    }
  }
  const range = max - min;
  const multiplier = 256 / range;
  for (let i = 0; i < image.data.length; i++) {
    const pixel = image.data[i];
    if (pixel === image.no_data_value) {
      output[i] = 0;
    } else {
      output[i] = (image.data[i] - min) * multiplier;
    }
  }
  return output;
}

export const mean = (arr: ArrayLike<number>): number => {
  let sum = 0;
  for (let i = 0; i < arr.length; i++) {
    sum = sum + arr[i];
  }
  return sum / arr.length;
};
