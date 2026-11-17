import { describe, expect, it } from "vitest";
import { BACKGROUND_PATTERNS, type DotInput } from "@/utils/backgroundPatterns";
import { gradientAt, parseColor, rotateHue, withAlpha } from "@/utils/color";
import { noise3 } from "@/utils/noise";

describe("bruit simplex", () => {
  it("reste dans [-1, 1] et redonne la même valeur au même point", () => {
    for (let i = 0; i < 2000; i++) {
      const value = noise3(i * 0.37, i * 0.11 - 40, i * 0.05);
      expect(value).toBeGreaterThanOrEqual(-1);
      expect(value).toBeLessThanOrEqual(1);
      expect(noise3(i * 0.37, i * 0.11 - 40, i * 0.05)).toBe(value);
    }
  });

  it("varie doucement d'un point à son voisin", () => {
    expect(Math.abs(noise3(1.2, 3.4, 5.6) - noise3(1.201, 3.4, 5.6))).toBeLessThan(0.05);
  });
});

describe("couleurs", () => {
  it("lit les couleurs du thème", () => {
    expect(parseColor("#4c5fef")).toEqual([76, 95, 239]);
    expect(parseColor("#fff")).toEqual([255, 255, 255]);
    expect(parseColor("rgba(236, 239, 255, 0.54)")).toEqual([236, 239, 255]);
  });

  it("rend une couleur translucide, sauf à l'opacité 1", () => {
    expect(withAlpha("#151a2e", 0.9)).toBe("rgba(21, 26, 46, 0.9)");
    expect(withAlpha("#151a2e", 1)).toBe("#151a2e");
  });

  it("tourne la teinte sans changer la couleur à 0° ni à 360°", () => {
    const accent = parseColor("#745af4");
    rotateHue("#745af4", 0).forEach((value, i) => expect(value).toBeCloseTo(accent[i], 0));
    rotateHue("#745af4", 360).forEach((value, i) => expect(value).toBeCloseTo(accent[i], 0));
  });

  it("tourne le violet de l'accent vers le magenta et vers le bleu", () => {
    const [r1, , b1] = rotateHue("#745af4", 40);
    const [r2, , b2] = rotateHue("#745af4", -40);
    expect(r1).toBeGreaterThan(b1 - 60);
    expect(b2).toBeGreaterThan(r2);
  });

  it("suit les arrêts d'un dégradé", () => {
    const stops = parseColor("#000000");
    expect(gradientAt([stops, [200, 100, 50]], 0.5)).toEqual([100, 50, 25]);
    expect(gradientAt([stops, [200, 100, 50]], 2)).toEqual([200, 100, 50]);
  });
});

describe("motifs du fond", () => {
  it("donnent des points de taille, de teinte et d'opacité valides", () => {
    const input: DotInput = {
      x: 0,
      y: 0,
      t: 3.2,
      now: 1.5,
      spacing: 6,
      isDark: true,
      origin: { x: 700, y: 40 },
    };
    Object.values(BACKGROUND_PATTERNS).forEach(pattern => {
      for (let i = 0; i < 400; i++) {
        input.x = (i * 37) % 800;
        input.y = (i * 53) % 600;
        const dot = pattern.dot(input);
        expect(dot.size).toBeGreaterThan(0);
        expect(dot.tone).toBeGreaterThanOrEqual(0);
        expect(dot.tone).toBeLessThanOrEqual(1);
        expect(dot.alpha).toBeGreaterThan(0);
      }
    });
  });
});

describe("indépendance vis-à-vis de la fenêtre", () => {
  it("fait avancer l'onde à la même vitesse dans toutes les fenêtres", () => {
    const at = (now: number, distance: number) =>
      BACKGROUND_PATTERNS.ripple.dot({
        x: 100 + distance,
        y: 100,
        t: 0,
        now,
        spacing: 4,
        isDark: true,
        origin: { x: 100, y: 100 },
      }).size;
    // Au bout d'une seconde, l'onde est à 400 px de la pastille, et pas à 200 ni à 600
    expect(at(1, 400)).toBeGreaterThan(at(1, 200));
    expect(at(1, 400)).toBeGreaterThan(at(1, 600));
  });
});
