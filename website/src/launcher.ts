import {
  VectorLayerLocator,
  RasterLayerLocator,
  InitialData,
  InitialVectorData,
} from "./types";
import { GisManager } from "./map-viewer";
import { speak, geoJsonParsers } from "touch-device";

const params = new URLSearchParams(location.search);
const vectorUrlFragment = params.get("vector");
const rasterUrl = params.get("raster");

const minLon = Number(params.get("min_lon") ?? -180);
const maxLon = Number(params.get("max_lon") ?? 180);
const minLat = Number(params.get("min_lat") ?? -90);
const maxLat = Number(params.get("max_lat") ?? 90);

const root = document.getElementById("image");

const vectorInputComponent = (url: string | null) => {
  const wrapper = document.createElement("div");
  const heading = document.createElement("h2");
  heading.innerText = "Vector";
  wrapper.appendChild(heading);
  const fileInput = labeledInput("file", "file");
  const urlInput = labeledInput("url", "url");
  const textInput = labeledInput("textarea", "paste or type geojson");
  const getData = (): VectorLayerLocator[] => {
    switch (activeInput) {
      case urlInput:
        if (urlInput.input.value.length > 0) {
          return [{ type: "url", url: urlInput.input.value }];
        } else {
          return [];
        }
      case textInput:
        return [{ type: "text", text: textInput.input.value }];
      case fileInput:
        const file = fileInput.input.files?.item(0);
        if (typeof file === "undefined" || file === null) {
          return [];
        }
        return [{ type: "file", file }];
      default:
        return [];
    }
  };

  const inputTypeSelector = document.createElement("select");
  inputTypeSelector.appendChild(new Option("Download from URL", "url", true));
  inputTypeSelector.appendChild(new Option("Upload file", "file"));
  inputTypeSelector.appendChild(new Option("Paste text", "text"));
  let activeInput: {
    label: HTMLLabelElement;
    input: HTMLInputElement | HTMLTextAreaElement;
  } = urlInput;
  urlInput.input.value = url ?? "";
  inputTypeSelector.addEventListener("change", (e) => {
    e.preventDefault();
    switch (inputTypeSelector.value) {
      case "url":
        activeInput.label.replaceWith(urlInput.label);
        activeInput = urlInput;
        break;
      case "file":
        activeInput.label.replaceWith(fileInput.label);
        activeInput = fileInput;
        break;
      case "text":
        activeInput.label.replaceWith(textInput.label);
        activeInput = textInput;
        break;
    }
  });
  wrapper.appendChild(inputTypeSelector);
  wrapper.appendChild(activeInput.label);
  return { parent: wrapper, getData };
};

const rasterInputComponent = (url: string | null) => {
  const wrapper = document.createElement("div");
  const heading = document.createElement("h2");
  heading.innerText = "Raster";
  wrapper.appendChild(heading);
  const fileInput = labeledInput("file", "file");
  const urlInput = labeledInput("url", "url");
  const getData = (): RasterLayerLocator | null => {
    switch (activeInput) {
      case urlInput:
        if (urlInput.input.value.length > 0) {
          return { type: "url", url: urlInput.input.value };
        } else {
          return null;
        }
      case fileInput:
        const file = fileInput.input.files?.item(0);
        if (typeof file === "undefined" || file === null) {
          return null;
        }
        return { type: "file", file };
      default:
        return null;
    }
  };

  const inputTypeSelector = document.createElement("select");
  inputTypeSelector.appendChild(new Option("Download from URL", "url", true));
  inputTypeSelector.appendChild(new Option("Upload file", "file"));
  let activeInput = urlInput;
  urlInput.input.value = url ?? "";
  inputTypeSelector.addEventListener("change", (e) => {
    e.preventDefault();
    switch (inputTypeSelector.value) {
      case "url":
        activeInput.label.replaceWith(urlInput.label);
        activeInput = urlInput;
        break;
      case "file":
        activeInput.label.replaceWith(fileInput.label);
        activeInput = fileInput;
        break;
    }
  });

  wrapper.appendChild(inputTypeSelector);
  wrapper.appendChild(activeInput.label);
  return { parent: wrapper, getData };
};

const labeledInput = <T extends string>(
  inputType: T,
  labelText: string
): {
  label: HTMLLabelElement;
  input: T extends "textarea" ? HTMLTextAreaElement : HTMLInputElement;
} => {
  const label = document.createElement("label");
  const input = document.createElement(
    inputType === "textarea" ? "textarea" : "input"
  );
  if (input instanceof HTMLInputElement) {
    input.type = inputType;
  }
  label.innerText = labelText;
  label.appendChild(input);
  return { label, input: input as unknown as any };
};

export function app() {
  const form = document.getElementById("options") as HTMLFormElement;
  const vectorInput = document.getElementById("vector") as HTMLDivElement;
  const rasterInput = document.getElementById("raster") as HTMLDivElement;
  const voiceInput = document.getElementById("voice") as HTMLSelectElement;

  const vectorUrl = decodeURIComponent(
    typeof vectorUrlFragment === "string" ? vectorUrlFragment : ""
  );
  const vectorInputContents = vectorInputComponent(vectorUrl);
  vectorInput.replaceWith(vectorInputContents.parent);
  const rasterInputContents = rasterInputComponent(
    typeof rasterUrl === "string" ? decodeURIComponent(rasterUrl) : ""
  );
  rasterInput.replaceWith(rasterInputContents.parent);

  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    // Speech needs to be ran on a explicit button click to allow it to work for other interactions to work on certain browsers
    const settings = {
      vector: vectorInputContents.getData(),
      raster: rasterInputContents.getData(),
      voice: voices[voiceInput.selectedIndex],
      audioTable: null,
      displaySettings: null,
    };
    const data = await loadData(settings);
    root?.replaceChildren();
    speak(
      "If you are using a screen reader please turn it off to use this application",
      settings.voice
    );
    new GisManager(data);
  });
  const synth = window.speechSynthesis;
  let voices = synth.getVoices();
  const updateVoices = () => {
    voices = synth.getVoices();
    // Clear current selection before repopulating it
    voiceInput.replaceChildren();
    for (let voice of voices) {
      const option = document.createElement("option");
      option.textContent = `${voice.name} (${voice.lang})`;
      if (voice.default) {
        if (voice.lang == window.navigator?.language) {
          option.defaultSelected = true;
        }
        option.textContent += " - Default";
      }
      voiceInput.appendChild(option);
    }
  };
  updateVoices();
  synth.addEventListener("voiceschanged", () => updateVoices());
}

type LauncherData = {
  vector: VectorLayerLocator[];
  raster: RasterLayerLocator | null;
  audioTable: string | null;
  displaySettings: string | null;
  voice: SpeechSynthesisVoice;
};

const loadData = async (settings: LauncherData): Promise<InitialData> => {
  const [vector, raster] = await Promise.all([
    loadVectorData(settings.vector),
    loadRasterData(settings.raster),
  ]);

  const bounds = {
    leftLon: minLon,
    rightLon: maxLon,
    topLat: maxLat,
    bottomLat: minLat,
  };

  return { ...settings, raster, vector, bounds: bounds };
};

const loadVectorData = async (
  location: VectorLayerLocator[]
): Promise<InitialVectorData[]> => {
  const displaySettings = null;
  const vectors = location.map(async (vector) => {
    switch (vector.type) {
      case "url": {
        const res = await fetch(vector.url);
        const json = await res.json();
        const features = geoJsonParsers.featureCollectionNonNull.parse(json);
        return { name: vector.url, features, displaySettings };
      }
      case "file": {
        const json = JSON.parse(await vector.file.text());
        const features = geoJsonParsers.featureCollectionNonNull.parse(json);
        return { name: vector.file.name, features, displaySettings };
      }
      case "text": {
        const json = JSON.parse(vector.text);
        const features = geoJsonParsers.featureCollectionNonNull.parse(json);
        return { name: "text", features, displaySettings };
      }
    }
  });
  return await Promise.all(vectors);
};

const loadRasterData = async (
  location: RasterLayerLocator | null
): Promise<ArrayBuffer | null> => {
  if (location === null) {
    return null;
  }

  switch (location.type) {
    case "url": {
      const res = await fetch(location.url);
      const data = await res.arrayBuffer();
      return data;
    }
    case "file": {
      const data = await location.file.arrayBuffer();
      return data;
    }
  }
};
