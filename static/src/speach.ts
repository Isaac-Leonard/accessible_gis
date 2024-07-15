const synth = window.speechSynthesis;
let voice: undefined | SpeechSynthesisVoice;
const updateVoices = () => {
  const newVoices = synth.getVoices();
  voice = newVoices.find((v) => v.name.toLowerCase() === "daniel");
};

if (typeof synth !== "undefined" && synth.onvoiceschanged !== undefined) {
  synth.onvoiceschanged = () => {
    updateVoices();
  };
}

// Initial call to updateVoices in case voices are already available
updateVoices();

export async function speak(text: string) {
  if (!voice) {
    return;
  }
  const announcement = new SpeechSynthesisUtterance(text);
  announcement.voice = voice;
  announcement.rate = 1;
  let res_: (value: unknown) => void;
  announcement.lang = voice.lang;
  const done = new Promise((res) => {
    res_ = res;
  });
  synth.cancel();
  announcement.addEventListener("end", () => res_(null));
  announcement.addEventListener("error", () => {
    res_(null);
  });
  announcement.addEventListener("start", () => {});
  announcement.volume = 1;
  synth.speak(announcement);
  await done;
}
