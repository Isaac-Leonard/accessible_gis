const audioCtx = new AudioContext();
audioCtx.suspend();
const wave = audioCtx.createOscillator();
wave.start();
const volume = audioCtx.createGain();
wave.connect(volume);
volume.connect(audioCtx.destination);
export const playAudio = async () => {
  await audioCtx.resume();
};

export const setAudioFrequency = (freq: number) => {
  wave.frequency.setValueAtTime(freq, audioCtx.currentTime);
};

export const pauseAudio = async () => {
  await audioCtx.suspend();
};
