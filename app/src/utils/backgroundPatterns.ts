import type { BaskulConfig } from "@/types/Config";
import type { Rgb } from "@/utils/color";
import { noise3 } from "@/utils/noise";

export type BackgroundName = Exclude<BaskulConfig["ui"]["background"], "none">;

/** Période de l'onde de connexion et de la pulsation de la pastille d'état */
export const PULSE_SECONDS = 3;
/** Distance parcourue par l'onde en une période (px) : la même quelle que soit la taille de la fenêtre */
const RIPPLE_REACH = 1200;

/** Écart de teinte (OKLCH) entre l'accent et ses deux voisines, les autres couleurs du fond */
export const NEIGHBOUR_HUE = 40;

export interface BackgroundPalette {
  neutral: Rgb;
  accent: Rgb;
  /** L'accent tourné de +NEIGHBOUR_HUE et de -NEIGHBOUR_HUE */
  secondary: Rgb;
  tertiary: Rgb;
  /** Couleur de l'état de la connexion, pour l'onde */
  pulse: Rgb;
}

export interface DotInput {
  /** Position par rapport au centre de la fenêtre (px) */
  x: number;
  y: number;
  /** Horloge du bruit (s), ralentie ou accélérée par la vitesse */
  t: number;
  /** Temps réel (s), qui rythme l'onde */
  now: number;
  spacing: number;
  isDark: boolean;
  /** Point de départ de l'onde, la pastille d'état, par rapport au centre de la fenêtre */
  origin: { x: number; y: number } | null;
}

export interface Dot {
  /** Rayon, en fraction du pas de la grille */
  size: number;
  /** Position dans les couleurs du motif, de 0 à 1 */
  tone: number;
  alpha: number;
}

export interface BackgroundPattern {
  stops: (palette: BackgroundPalette) => Rgb[];
  dot: (input: DotInput) => Dot;
}

function smooth(edge0: number, edge1: number, value: number) {
  const ratio = Math.min(1, Math.max(0, (value - edge0) / (edge1 - edge0)));
  return ratio * ratio * (3 - 2 * ratio);
}

function unitNoise(x: number, y: number, z: number) {
  return (noise3(x, y, z) + 1) / 2;
}

function rippleRing({ x, y, now, spacing, origin }: DotInput) {
  let ring = 0;
  if (origin) {
    const phase = (now % PULSE_SECONDS) / PULSE_SECONDS;
    const radius = phase * RIPPLE_REACH;
    const distance = Math.hypot(x - origin.x, y - origin.y) - radius;
    const thickness = spacing * 4;
    ring = Math.exp(-(distance * distance) / (2 * thickness * thickness)) * (1 - phase);
  }
  return ring;
}

const wave: BackgroundPattern = {
  stops: palette => [palette.accent, palette.tertiary, palette.secondary],
  dot({ x, y, t, isDark }) {
    const glow = smooth(0.35, 0.95, unitNoise(x * 0.0017, y * 0.0017, t * 0.22));
    return {
      size: 0.07 + glow * 0.12,
      tone: unitNoise(x * 0.0006 + 40, y * 0.0006, t * 0.12),
      alpha: (isDark ? 0.12 : 0.14) + glow * (isDark ? 0.75 : 0.62),
    };
  },
};

const breathing: BackgroundPattern = {
  stops: palette => [palette.neutral, palette.accent],
  dot({ x, y, t, isDark }) {
    const crest = smooth(0.62, 0.9, unitNoise(x * 0.0012, y * 0.0012, t * 0.16));
    return { size: 0.075 + crest * 0.05, tone: crest, alpha: (isDark ? 0.22 : 0.3) + crest * 0.55 };
  },
};

const current: BackgroundPattern = {
  stops: palette => [palette.accent, palette.secondary],
  dot({ x, y, t, isDark }) {
    const nx = x * 0.0014 - t * 0.072;
    const ny = y * 0.0014 + t * 0.036;
    const band = smooth(0.4, 0.85, unitNoise(nx, ny, t * 0.1));
    return {
      size: 0.05 + band * 0.2,
      tone: smooth(0.3, 0.9, unitNoise(nx * 0.5, ny * 0.5, 9)),
      alpha: (isDark ? 0.1 : 0.12) + band * (isDark ? 0.7 : 0.55),
    };
  },
};

// Neutre vers l'accent pour le bruit, puis vers la couleur de l'état pour l'onde
const ripple: BackgroundPattern = {
  stops: palette => [palette.neutral, palette.accent, palette.pulse],
  dot(input) {
    const { x, y, t, isDark } = input;
    const base = smooth(0.55, 0.95, unitNoise(x * 0.0016, y * 0.0016, t * 0.14)) * 0.5;
    const ring = rippleRing(input);
    return {
      size: 0.05 + ring * 0.22,
      tone: ring > 0.05 ? 0.5 + ring * 0.5 : base,
      alpha: (isDark ? 0.18 : 0.24) + Math.min(1, ring + base) * 0.6,
    };
  },
};

export const BACKGROUND_PATTERNS: Record<BackgroundName, BackgroundPattern> = {
  wave,
  breathing,
  current,
  ripple,
};
