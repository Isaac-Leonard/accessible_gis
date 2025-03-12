import { OcrEngine, OcrEngineInit, default as initOcrLib } from "./ocrs";

export const renderAccessibleImageViewer = async (
  canvas: HTMLCanvasElement,
  image: ImageData
) => {
  const context = canvas.getContext("2d")!;
  context.putImageData(image, 0, 0);
};

type OCRResources = {
  detectionModel: Uint8Array;
  recognitionModel: Uint8Array;
};
let ocrResources: Promise<OCRResources> | undefined;

/**
 * Create an OCR engine and configure its models.
 */
async function createOCREngine(): Promise<OcrEngine> {
  if (!ocrResources) {
    // Initialize OCR library and fetch models on first use.
    const init = async () => {
      const [ocrBin, detectionModel, recognitionModel] = await Promise.all([
        fetch("/ocrs_bg.wasm").then((r) => r.arrayBuffer()),
        fetch("/text-detection.rten").then((r) => r.arrayBuffer()),
        fetch("/text-recognition.rten").then((r) => r.arrayBuffer()),
      ]);

      await initOcrLib(ocrBin);

      return {
        detectionModel: new Uint8Array(detectionModel),
        recognitionModel: new Uint8Array(recognitionModel),
      };
    };
    ocrResources = init();
  }

  const { detectionModel, recognitionModel } = await ocrResources;
  const ocrInit = new OcrEngineInit();
  ocrInit.setDetectionModel(detectionModel);
  ocrInit.setRecognitionModel(recognitionModel);
  return new OcrEngine(ocrInit);
}

const ocrEngine = await createOCREngine();

export const getTextFromImage = (image: ImageData) => {
  const ocrImage = ocrEngine.loadImage(
    image.width,
    image.height,
    Uint8Array.from(image.data)
  );
  const lineLocations = ocrEngine.detectText(ocrImage);
  const lineRects = lineLocations.map((x) =>
    domRectFromRotatedRect(Array.from(x.rotatedRect().corners()))
  );
  return ocrEngine.recognizeText(ocrImage, lineLocations).map((line, i) => ({
    text: line.text(),
    rect: lineRects[i],
  }));
};

type RotatedRect = number[];
/**
 * Return the smallest axis-aligned rect that contains all corners of a
 * rotated rect.
 */
function domRectFromRotatedRect(coords: RotatedRect): DOMRect {
  const [x0, y0, x1, y1, x2, y2, x3, y3] = coords;
  const left = Math.min(x0, x1, x2, x3);
  const top = Math.min(y0, y1, y2, y3);
  const right = Math.max(x0, x1, x2, x3);
  const bottom = Math.max(y0, y1, y2, y3);
  return new DOMRect(left, top, right - left, bottom - top);
}
