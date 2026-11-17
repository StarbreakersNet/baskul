<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, useTemplateRef, watch } from "vue";
import { useThemeVars } from "naive-ui";
import { useBackgroundTuningStore } from "@/stores/backgroundTuning";
import {
  BACKGROUND_PATTERNS,
  NEIGHBOUR_HUE,
  type BackgroundName,
  type BackgroundPalette,
  type DotInput,
} from "@/utils/backgroundPatterns";
import { gradientAt, parseColor, rgbaString, rotateHue } from "@/utils/color";

const {
  pattern,
  origin = null,
  pulseColor,
} = defineProps<{
  pattern: BackgroundName;
  /** Élément d'où part l'onde de connexion */
  origin?: HTMLElement | null;
  pulseColor: string;
}>();

const FRAME_MS = 40;
// Cadence ralentie si une image coûte plus que cette part du temps entre deux images (grand écran)
const MAX_DRAW_SHARE = 0.4;
const MAX_FRAME_MS = 120;
const MAX_PIXEL_RATIO = 2;
const MIN_RADIUS = 0.4;
// Les points sont regroupés par teinte et par opacité : un seul remplissage par groupe
const TONE_STEPS = 12;
const ALPHA_STEPS = 16;

const canvasRef = useTemplateRef("canvasRef");
const themeVars = useThemeVars();
const backgroundTuning = useBackgroundTuningStore();
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const isDark = computed(() => {
  const [r, g, b] = parseColor(themeVars.value.bodyColor);
  return r + g + b < 384;
});

let palette = readPalette();
let frameId = 0;
let clock = 0;
let lastTick = 0;
let lastDraw = 0;
let frameInterval = FRAME_MS;
let isResizePending = false;

function readPalette(): BackgroundPalette {
  return {
    neutral: parseColor(themeVars.value.textColor3),
    accent: parseColor(themeVars.value.primaryColor),
    secondary: rotateHue(themeVars.value.primaryColor, NEIGHBOUR_HUE),
    tertiary: rotateHue(themeVars.value.primaryColor, -NEIGHBOUR_HUE),
    pulse: parseColor(pulseColor),
  };
}

function originPoint(centerX: number, centerY: number) {
  const rect = origin?.getBoundingClientRect();
  return rect
    ? { x: rect.left + rect.height / 2 - centerX, y: rect.top + rect.height / 2 - centerY }
    : null;
}

function resize() {
  const canvas = canvasRef.value;
  if (canvas) {
    const ratio = Math.min(MAX_PIXEL_RATIO, window.devicePixelRatio || 1);
    canvas.width = Math.round(window.innerWidth * ratio);
    canvas.height = Math.round(window.innerHeight * ratio);
    // Échelle exacte après l'arrondi : un pixel CSS tombe toujours au même endroit du canevas
    canvas
      .getContext("2d")
      ?.setTransform(
        canvas.width / window.innerWidth,
        0,
        0,
        canvas.height / window.innerHeight,
        0,
        0
      );
  }
}

function addDot(paths: Map<number, Path2D>, input: DotInput, centerX: number, centerY: number) {
  const dot = BACKGROUND_PATTERNS[pattern].dot(input);
  const alpha = Math.round(Math.min(1, dot.alpha) * (ALPHA_STEPS - 1));
  if (alpha > 0) {
    const key = Math.round(dot.tone * (TONE_STEPS - 1)) * ALPHA_STEPS + alpha;
    const path = paths.get(key) ?? new Path2D();
    const radius = Math.max(MIN_RADIUS, dot.size * input.spacing);
    const x = input.x + centerX;
    const y = input.y + centerY;
    if (radius < 1) {
      path.rect(x - radius, y - radius, radius * 2, radius * 2);
    } else {
      path.moveTo(x + radius, y);
      path.arc(x, y, radius, 0, Math.PI * 2);
    }
    paths.set(key, path);
  }
}

function fillPaths(context: CanvasRenderingContext2D, paths: Map<number, Path2D>) {
  const stops = BACKGROUND_PATTERNS[pattern].stops(palette);
  paths.forEach((path, key) => {
    const tone = Math.floor(key / ALPHA_STEPS) / (TONE_STEPS - 1);
    context.fillStyle = rgbaString(
      gradientAt(stops, tone),
      (key % ALPHA_STEPS) / (ALPHA_STEPS - 1)
    );
    context.fill(path);
  });
}

function draw() {
  const context = canvasRef.value?.getContext("2d");
  if (context) {
    const width = window.innerWidth;
    const height = window.innerHeight;
    const spacing = backgroundTuning.tuning.spacing;
    // Grille et motif partent du centre de la fenêtre, arrondi au pixel pour garder des points nets
    const centerX = Math.round(width / 2);
    const centerY = Math.round(height / 2);
    const paths = new Map<number, Path2D>();
    const input: DotInput = {
      x: 0,
      y: 0,
      t: clock,
      now: performance.now() / 1000,
      spacing,
      isDark: isDark.value,
      origin: originPoint(centerX, centerY),
    };
    for (let y = -Math.floor(centerY / spacing) * spacing; y < height - centerY; y += spacing) {
      for (let x = -Math.floor(centerX / spacing) * spacing; x < width - centerX; x += spacing) {
        input.x = x;
        input.y = y;
        addDot(paths, input, centerX, centerY);
      }
    }
    context.clearRect(0, 0, width, height);
    fillPaths(context, paths);
  }
}

function tick(time: number) {
  if (time - lastDraw >= frameInterval) {
    clock += (Math.min(MAX_FRAME_MS, time - lastTick) / 1000) * backgroundTuning.tuning.speed;
    lastTick = time;
    lastDraw = time;
    const start = performance.now();
    draw();
    const cost = performance.now() - start;
    frameInterval = Math.min(MAX_FRAME_MS, Math.max(FRAME_MS, cost / MAX_DRAW_SHARE));
  }
  frameId = requestAnimationFrame(tick);
}

function stop() {
  cancelAnimationFrame(frameId);
  frameId = 0;
}

// Arrêté quand la fenêtre est cachée ; une seule image, fixe, si le système demande moins de mouvement
function refresh() {
  stop();
  if (document.hidden || reducedMotion.matches) {
    draw();
  } else {
    lastTick = performance.now();
    frameId = requestAnimationFrame(tick);
  }
}

// Une fenêtre qu'on étire envoie des dizaines d'événements : un seul nouveau dessin par image
function handleResize() {
  if (!isResizePending) {
    isResizePending = true;
    requestAnimationFrame(() => {
      isResizePending = false;
      resize();
      draw();
    });
  }
}

watch([themeVars, () => pulseColor], () => {
  palette = readPalette();
  draw();
});

watch([() => pattern, () => backgroundTuning.tuning.spacing], () => draw());

onMounted(() => {
  resize();
  refresh();
  window.addEventListener("resize", handleResize);
  document.addEventListener("visibilitychange", refresh);
  reducedMotion.addEventListener("change", refresh);
});

onBeforeUnmount(() => {
  stop();
  window.removeEventListener("resize", handleResize);
  document.removeEventListener("visibilitychange", refresh);
  reducedMotion.removeEventListener("change", refresh);
});
</script>

<template>
  <canvas ref="canvasRef" aria-hidden="true" class="app-background" />
</template>

<style lang="scss" scoped>
.app-background {
  position: fixed;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>
