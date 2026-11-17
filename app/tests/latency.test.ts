import { describe, expect, it } from "vitest";
import { formatMs, latencyStats, linkQuality, sparklinePoints } from "@/utils/latency";

const samples = (values: number[]) => values.map((ms, index) => ({ at: index * 2000, ms }));

describe("diagnostic réseau", () => {
  it("juge la qualité du lien sur les dernières mesures", () => {
    expect(linkQuality(2)).toBe("good");
    expect(linkQuality(15)).toBe("fair");
    expect(linkQuality(45)).toBe("poor");
    // Un pic ancien ne suffit pas à dégrader le lien
    expect(latencyStats(samples([80, 2, 2, 2, 2, 2]))?.quality).toBe("good");
    expect(latencyStats(samples([2, 40, 50, 45, 60, 55]))?.quality).toBe("poor");
  });

  it("calcule la dernière mesure, la moyenne et le maximum", () => {
    expect(latencyStats(samples([1, 2, 3, 6]))).toEqual({
      last: 6,
      average: 3,
      max: 6,
      quality: "good",
    });
    expect(latencyStats([])).toBeNull();
  });

  it("trace la courbe dans le cadre", () => {
    expect(sparklinePoints(samples([0, 30]), 100, 20)).toBe("0,20 100,0");
    expect(sparklinePoints(samples([15]), 100, 20)).toBe("0,10");
  });

  it("affiche une décimale sous 10 ms", () => {
    expect(formatMs(1.234)).toBe("1.2");
    expect(formatMs(12.6)).toBe("13");
  });
});
