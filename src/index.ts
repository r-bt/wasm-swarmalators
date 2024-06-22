import init, { Swarmalator } from "my-crate";
import { memory } from "my-crate/my_crate_bg";
// import { memory } from "my-create/my_crate_bg";
// Don't worry if vscode told you can't find my-crate
// It's because you're using a local crate
// after yarn dev, wasm-pack plugin will install my-crate for you

function mapRange(num, inMin, inMax, outMin, outMax) {
  return ((num - inMin) * (outMax - outMin)) / (inMax - inMin) + outMin;
}

function radiansToHSV(angleInRadians) {
  // Convert radians to degrees
  let degrees = angleInRadians * (180 / Math.PI);

  // Normalize degrees to [0, 360)
  let hue = ((degrees % 360) + 360) % 360;

  // Set saturation and value to maximum
  let saturation = 1;
  let value = 1;

  return { hue, saturation, value };
}

function HSVtoRGB(h, s, v) {
  let c = v * s;
  let x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  let m = v - c;
  let r, g, b;

  if (0 <= h && h < 60) {
    r = c;
    g = x;
    b = 0;
  } else if (60 <= h && h < 120) {
    r = x;
    g = c;
    b = 0;
  } else if (120 <= h && h < 180) {
    r = 0;
    g = c;
    b = x;
  } else if (180 <= h && h < 240) {
    r = 0;
    g = x;
    b = c;
  } else if (240 <= h && h < 300) {
    r = x;
    g = 0;
    b = c;
  } else {
    r = c;
    g = 0;
    b = x;
  }

  r = Math.round((r + m) * 255);
  g = Math.round((g + m) * 255);
  b = Math.round((b + m) * 255);

  return { r, g, b };
}

function HSVtoCanvasFillStyle(angleInRadians) {
  let hsv = radiansToHSV(angleInRadians);
  let rgb = HSVtoRGB(hsv.hue, hsv.saturation, hsv.value);

  return `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
}

init().then(({ memory }) => {
  const agents = 10;

  // Create random positions
  const agent_positions = Array.from({ length: agents }, () => [
    Math.random() * 6 - 3,
    Math.random() * 6 - 3,
  ]);

  // Lin space the phase
  const agent_phases = Array.from(
    { length: agents },
    (_, i) => (i / agents) * 2 * Math.PI
  );
  // const agent_phases = Array.from(
  //   { length: agents },
  //   () => Math.random() * 2 * Math.PI
  // );

  // Natural frequencies, first half 1 next -1
  // const natural_frequencies = Array.from({ length: agents }, (_, i) =>
  //   i < agents / 2 ? 1 : -1
  // );
  const natural_frequencies = Array.from({ length: agents }, () => 0);

  // Chiral coefficents, first half 1 next -1
  const chiral_coefficients = Array.from({ length: agents }, (_, i) =>
    i < agents / 2 ? 1 : -1
  );

  const target = [2, 0];

  const float64_positions = new Float64Array(agent_positions.flat());
  const float64_phases = new Float64Array(agent_phases);
  const float64_natural_frequencies = new Float64Array(natural_frequencies);
  const float64_chiral_coefficients = new Float64Array(chiral_coefficients);
  const float64_target = new Float64Array(target);

  const swarmalator = new Swarmalator(
    agents,
    float64_positions,
    float64_phases,
    float64_natural_frequencies,
    1,
    0,
    undefined,
    float64_target
  );

  const canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
  if (!canvas) {
    console.error("Canvas element not found");
    return;
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    console.error("2D context not available");
    return;
  }

  // Set display size (css pixels).
  const size = 800;
  canvas.style.width = `${size}px`;
  canvas.style.height = `${size}px`;

  // Set actual size in memory (scaled to account for extra pixel density).
  const scale = window.devicePixelRatio; // Change to 1 on retina screens to see blurry canvas.
  canvas.width = Math.floor(size * scale);
  canvas.height = Math.floor(size * scale);

  ctx.scale(scale, scale);

  let count = 0;

  function updateAndDraw() {
    swarmalator.update(0.05);

    count += 1;

    const positionsPtr = swarmalator.positions();
    const positions = new Float64Array(memory.buffer, positionsPtr, agents * 2);

    const velocitiesPtr = swarmalator.velocities();
    const velocities = new Float64Array(
      memory.buffer,
      velocitiesPtr,
      agents * 2
    );

    const phasesPtr = swarmalator.phases();
    const phases = new Float64Array(memory.buffer, phasesPtr, agents);

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (let i = 0; i < agents; i++) {
      const x = mapRange(positions[i * 2], -3, 3, 0.1 * size, 0.9 * size);
      const y = mapRange(positions[i * 2 + 1], -3, 3, 0.9 * size, 0.1 * size);

      ctx.beginPath();
      ctx.arc(x, y, 5, 0, 2 * Math.PI);

      const phase = phases[i];

      ctx.fillStyle = HSVtoCanvasFillStyle(phase);
      ctx.fill();
    }

    requestAnimationFrame(updateAndDraw);
  }

  updateAndDraw();
});
