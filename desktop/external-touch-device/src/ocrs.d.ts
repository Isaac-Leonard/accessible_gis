/* tslint:disable */
/* eslint-disable */
/**
* A line of text that has been detected, but not recognized.
*
* This contains information about the location of the text, but not the
* string contents.
*/
export class DetectedLine {
  free(): void;
/**
* @returns {RotatedRect}
*/
  rotatedRect(): RotatedRect;
/**
* @returns {(RotatedRect)[]}
*/
  words(): (RotatedRect)[];
}
/**
* A pre-processed image that can be passed as input to `OcrEngine.loadImage`.
*/
export class Image {
  free(): void;
/**
* Return the number of channels in the image.
* @returns {number}
*/
  channels(): number;
/**
* Return the width of the image.
* @returns {number}
*/
  width(): number;
/**
* Return the height of the image.
* @returns {number}
*/
  height(): number;
/**
* Return the image data in row-major, channels-last order.
* @returns {Uint8Array}
*/
  data(): Uint8Array;
}
/**
* OcrEngine is the main API for performing OCR in WebAssembly.
*/
export class OcrEngine {
  free(): void;
/**
* Construct a new `OcrEngine` using the models and other settings given
* by `init`.
*
* To detect text in an image, `init` must have a detection model set.
* To recognize text, `init` must have a recognition model set.
* @param {OcrEngineInit} init
*/
  constructor(init: OcrEngineInit);
/**
* Prepare an image for analysis by the OCR engine.
*
* The image is an array of pixels in row-major, channels last order. This
* matches the format of the
* [ImageData](https://developer.mozilla.org/en-US/docs/Web/API/ImageData)
* API. Supported channel combinations are RGB and RGBA. The number of
* channels is inferred from the length of `data`.
* @param {number} width
* @param {number} height
* @param {Uint8Array} data
* @returns {Image}
*/
  loadImage(width: number, height: number, data: Uint8Array): Image;
/**
* Detect text in an image.
*
* Returns a list of lines that were found. These can be passed to
* `recognizeText` identify the characters.
* @param {Image} image
* @returns {(DetectedLine)[]}
*/
  detectText(image: Image): (DetectedLine)[];
/**
* Recognize text that was previously detected with `detectText`.
*
* Returns a list of `TextLine` objects that can be used to query the text
* and bounding boxes of each line.
* @param {Image} image
* @param {(DetectedLine)[]} lines
* @returns {(TextLine)[]}
*/
  recognizeText(image: Image, lines: (DetectedLine)[]): (TextLine)[];
/**
* Detect and recognize text in an image.
*
* Returns a single string containing all the text found in reading order.
* @param {Image} image
* @returns {string}
*/
  getText(image: Image): string;
/**
* Detect and recognize text in an image.
*
* Returns a list of `TextLine` objects that can be used to query the text
* and bounding boxes of each line.
* @param {Image} image
* @returns {(TextLine)[]}
*/
  getTextLines(image: Image): (TextLine)[];
}
/**
* Options for constructing an [OcrEngine].
*/
export class OcrEngineInit {
  free(): void;
/**
*/
  constructor();
/**
* Load a model for text detection.
* @param {Uint8Array} data
*/
  setDetectionModel(data: Uint8Array): void;
/**
* Load a model for text recognition.
* @param {Uint8Array} data
*/
  setRecognitionModel(data: Uint8Array): void;
}
/**
*/
export class RotatedRect {
  free(): void;
/**
* Return an array of the X and Y coordinates of corners of this rectangle,
* arranged as `[x0, y0, ... x3, y3]`.
* @returns {Float32Array}
*/
  corners(): Float32Array;
/**
* Return the coordinates of the axis-aligned bounding rectangle of this
* rotated rect.
*
* The result is a `[left, top, right, bottom]` array of coordinates.
* @returns {Float32Array}
*/
  boundingRect(): Float32Array;
}
/**
* A sequence of `TextWord`s that were recognized, forming a line.
*/
export class TextLine {
  free(): void;
/**
* @returns {string}
*/
  text(): string;
/**
* @returns {(TextWord)[]}
*/
  words(): (TextWord)[];
}
/**
* Bounding box and text of a word that was recognized.
*/
export class TextWord {
  free(): void;
/**
* @returns {string}
*/
  text(): string;
/**
* Return the oriented bounding rectangle containing the characters in
* this word.
* @returns {RotatedRect}
*/
  rotatedRect(): RotatedRect;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_ocrengineinit_free: (a: number) => void;
  readonly ocrengineinit_new: () => number;
  readonly ocrengineinit_setDetectionModel: (a: number, b: number, c: number, d: number) => void;
  readonly ocrengineinit_setRecognitionModel: (a: number, b: number, c: number, d: number) => void;
  readonly __wbg_ocrengine_free: (a: number) => void;
  readonly ocrengine_new: (a: number, b: number) => void;
  readonly ocrengine_loadImage: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly ocrengine_detectText: (a: number, b: number, c: number) => void;
  readonly ocrengine_recognizeText: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly ocrengine_getText: (a: number, b: number, c: number) => void;
  readonly ocrengine_getTextLines: (a: number, b: number, c: number) => void;
  readonly __wbg_image_free: (a: number) => void;
  readonly image_channels: (a: number) => number;
  readonly image_width: (a: number) => number;
  readonly image_height: (a: number) => number;
  readonly image_data: (a: number, b: number) => void;
  readonly __wbg_rotatedrect_free: (a: number) => void;
  readonly rotatedrect_corners: (a: number, b: number) => void;
  readonly rotatedrect_boundingRect: (a: number, b: number) => void;
  readonly __wbg_detectedline_free: (a: number) => void;
  readonly detectedline_rotatedRect: (a: number) => number;
  readonly detectedline_words: (a: number, b: number) => void;
  readonly __wbg_textword_free: (a: number) => void;
  readonly textword_text: (a: number, b: number) => void;
  readonly textword_rotatedRect: (a: number) => number;
  readonly __wbg_textline_free: (a: number) => void;
  readonly textline_text: (a: number, b: number) => void;
  readonly textline_words: (a: number, b: number) => void;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __externref_drop_slice: (a: number, b: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
