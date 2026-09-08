import { speak } from "./speach.ts";
import { GestureManager } from "./touch.ts";

type ScreenReaderState = { speaking: SpeakingState; queue: string[] };

type SpeakingState = { isSpeaking: true; text: string } | { isSpeaking: false };

export class ScreenReader {
  state: ScreenReaderState = { speaking: { isSpeaking: false }, queue: [] };
  root: HTMLHtmlElement;
  gestureManager: GestureManager;

  focusedNode: Node;

  constructor(root: HTMLHtmlElement) {
    this.root = root;
    this.focusedNode = root;
    while (this.focusedNode.firstChild !== null) {
      this.focusedNode = this.focusedNode.firstChild;
    }
    this.gestureManager = new GestureManager(root);
    this.attachEventHandlers();
  }

  attachEventHandlers() {
    this.gestureManager.addTapHandler((target) => {
      this.speak(target);
    });

    this.gestureManager.addOneFingerSwipeHandler("right", () => {
      const nextNode = this.focusedNode.nextSibling;
      if (nextNode !== null) {
        this.focusedNode = nextNode;
        this.speak(this.focusedNode);
      } else {
        // TODO Actually handle this case properly
        throw new Error("no content to read error,no next sibling");
      }
    });

    this.gestureManager.addOneFingerSwipeHandler("left", () => {
      const prevNode = this.focusedNode.previousSibling;
      if (prevNode !== null) {
        this.focusedNode = prevNode;
        this.speak(this.focusedNode);
      } else {
        // TODO Actually handle this case properly
        throw new Error("no content to read error,no next sibling");
      }
    });

    this.gestureManager.addDoubleTapHandler(() => {
      if (this.focusedNode instanceof HTMLHtmlElement) {
        this.focusedNode.click();
      }
      {
      }
    });
  }

  speak(content: string | HTMLHtmlElement | Node) {
    if (typeof content === "string") {
      this.speakText(content);
    } else if (content instanceof HTMLHtmlElement) {
      const text = content.innerText;
      const contentType = content.tagName;
      const textToSpeak = `${text}\n${contentType}`;
      this.speakText(textToSpeak);
    } else {
      const text = content.textContent ?? "";
      this.speakText(text);
    }
  }

  async speakText(content: string) {
    this.state.speaking = { isSpeaking: true, text: content };
    await speak(content);
    this.state.speaking = { isSpeaking: false };
  }
}
