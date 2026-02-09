const synth = window.speechSynthesis;
let voice: SpeechSynthesisVoice;
const updateVoices = () => {
  const newVoices = synth.getVoices();
  const foundVoice = newVoices.find((v) =>
    v.name.toLowerCase().includes("daniel")
  );
  voice = foundVoice ?? newVoices[0];
};

synth.addEventListener("voiceschanged", () => {
  updateVoices();
});

// Initial call to updateVoices in case voices are already available
updateVoices();

export async function speak(
  text: string,
  preferedVoice?: SpeechSynthesisVoice
) {
  preferedVoice = preferedVoice ?? voice;
  if (!preferedVoice) {
    return;
  }
  const announcement = new SpeechSynthesisUtterance(text);
  announcement.voice = preferedVoice;
  synth.cancel();
  const done = new Promise((res) => {
    announcement.addEventListener("end", () => res(null));
    announcement.addEventListener("error", () => {
      res(null);
    });
  });
  announcement.volume = 1;
  announcement.rate = 1;
  announcement.lang = preferedVoice.lang;
  synth.speak(announcement);
  await done;
}
