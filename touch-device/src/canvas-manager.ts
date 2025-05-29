export function getCanvas() {
  const canvas = document.getElementById("canvas");
  if (!(canvas instanceof HTMLCanvasElement)) {
    const canvas = document.createElement("canvas")!;
    const container = document.getElementById("image");
    container!.appendChild(canvas);
    canvas.width = Math.max(
      document.documentElement.clientWidth,
      window.innerWidth
    );
    canvas.height = Math.max(
      document.documentElement.clientHeight,
      window.innerHeight
    );
    const ctx = canvas.getContext("2d");
    if (ctx == null) {
      throw new Error("Could not initialise canvas");
    }
    ctx.fillStyle = "#000000";
    ctx.strokeStyle = "#ffffff";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    return { canvas, ctx };
  }
  const ctx = canvas.getContext("2d");
  if (ctx == null) {
    throw new Error("Could not initialise canvas");
  }
  return { canvas, ctx };
}
