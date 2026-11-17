import type { LatencySample } from "@/types/Latency";

export type LinkQuality = "good" | "fair" | "poor";

export interface LatencyStats {
  last: number;
  average: number;
  max: number;
  quality: LinkQuality;
}

/** Au-delà, le curseur commence à se sentir en retard */
const FAIR_MS = 10;
const POOR_MS = 30;
/** La qualité se juge sur les dernières mesures, pas sur un pic isolé */
const RECENT = 5;

export function linkQuality(ms: number): LinkQuality {
  if (ms >= POOR_MS) {
    return "poor";
  }
  return ms >= FAIR_MS ? "fair" : "good";
}

export function latencyStats(samples: LatencySample[]): LatencyStats | null {
  let stats: LatencyStats | null = null;
  if (samples.length) {
    const values = samples.map(sample => sample.ms);
    const recent = values.slice(-RECENT);
    const recentAverage = recent.reduce((sum, ms) => sum + ms, 0) / recent.length;
    stats = {
      last: values[values.length - 1],
      average: values.reduce((sum, ms) => sum + ms, 0) / values.length,
      max: Math.max(...values),
      quality: linkQuality(recentAverage),
    };
  }
  return stats;
}

/** Points d'une courbe SVG (`polyline`) dans un cadre `width` × `height` */
export function sparklinePoints(samples: LatencySample[], width: number, height: number) {
  const max = Math.max(POOR_MS, ...samples.map(sample => sample.ms));
  const step = samples.length > 1 ? width / (samples.length - 1) : 0;
  return samples
    .map((sample, index) => {
      const x = Math.round(index * step * 10) / 10;
      const y = Math.round((height - (sample.ms / max) * height) * 10) / 10;
      return `${x},${y}`;
    })
    .join(" ");
}

export function formatMs(ms: number) {
  return ms < 10 ? ms.toFixed(1) : Math.round(ms).toString();
}
