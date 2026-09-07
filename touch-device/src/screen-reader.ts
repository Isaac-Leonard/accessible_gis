import { speak } from "./speach.ts";
import { GestureManager } from "./touch.ts";

type ScreenReaderState = { speaking: SpeakingState; queue: string[] };

type SpeakingState = { isSpeaking: true; text: string } | { isSpeaking: false };

export class ScreenReader {
  state: ScreenReaderState = { speaking: { isSpeaking: false }, queue: [] };
  root: HTMLHtmlElement;
  gestureManager: GestureManager;

  constructor(root: HTMLHtmlElement) {
    this.root = root;
    this.gestureManager = new GestureManager(root);
    this.attachEventHandlers();
  }

  attachEventHandlers() {
    this.gestureManager.addTapHandler((target) => {
      this.speak(target);
    });
  }

  speak(content: string | HTMLHtmlElement) {
    if (typeof content === "string") {
      this.speakText(content);
    } else {
      // TODO transform DOM structure into speakable text
    }
  }

  async speakText(content: string) {
    this.state.speaking = { isSpeaking: true, text: content };
    await speak(content);
    this.state.speaking = { isSpeaking: false };
  }
}
