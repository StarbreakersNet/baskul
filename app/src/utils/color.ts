export type Rgb = [number, number, number];

/** `#rgb`, `#rrggbb`, `rgb(…)` ou `rgba(…)` ; noir pour une couleur illisible */
export function parseColor(color: string): Rgb {
  const value = color.trim();
  if (value.startsWith("#")) {
    const hex =
      value.length === 4 ? Array.from(value.slice(1), c => c + c).join("") : value.slice(1, 7);
    return [0, 2, 4].map(start => parseInt(hex.slice(start, start + 2), 16) || 0) as Rgb;
  }
  const channels =
    value
      .match(/[\d.]+/g)
      ?.slice(0, 3)
      .map(Number) ?? [];
  return channels.length === 3 ? (channels as Rgb) : [0, 0, 0];
}

export function mixRgb(a: Rgb, b: Rgb, ratio: number): Rgb {
  return [0, 1, 2].map(i => a[i] + (b[i] - a[i]) * ratio) as Rgb;
}

/** Couleur le long d'un dégradé à plusieurs arrêts, `position` dans [0, 1] */
export function gradientAt(stops: Rgb[], position: number): Rgb {
  if (stops.length === 1) {
    return stops[0];
  }
  const scaled = Math.min(1, Math.max(0, position)) * (stops.length - 1);
  const index = Math.min(stops.length - 2, Math.floor(scaled));
  return mixRgb(stops[index], stops[index + 1], scaled - index);
}

export function rgbaString([r, g, b]: Rgb, alpha: number) {
  return `rgba(${Math.round(r)}, ${Math.round(g)}, ${Math.round(b)}, ${alpha})`;
}

/** La même couleur, rendue translucide */
export function withAlpha(color: string, alpha: number) {
  return alpha >= 1 ? color : rgbaString(parseColor(color), alpha);
}

function toLinear(channel: number) {
  const value = channel / 255;
  return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
}

function fromLinear(value: number) {
  return 255 * (value <= 0.0031308 ? 12.92 * value : 1.055 * value ** (1 / 2.4) - 0.055);
}

/** OKLab (Björn Ottosson) : la teinte y tourne sans changer la luminosité perçue */
function toOklab([r, g, b]: Rgb): Rgb {
  const [lr, lg, lb] = [r, g, b].map(toLinear);
  const l = Math.cbrt(0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb);
  const m = Math.cbrt(0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb);
  const s = Math.cbrt(0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function fromOklab([lightness, a, b]: Rgb): Rgb {
  const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ].map(value => Math.min(255, Math.max(0, fromLinear(value)))) as Rgb;
}

/** La même couleur, teinte tournée de `degrees` (OKLCH) : luminosité et intensité gardées */
export function rotateHue(color: string, degrees: number): Rgb {
  const [lightness, a, b] = toOklab(parseColor(color));
  const angle = (degrees * Math.PI) / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  return fromOklab([lightness, a * cos - b * sin, a * sin + b * cos]);
}
