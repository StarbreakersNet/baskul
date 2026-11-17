import type { GlobalThemeOverrides } from "naive-ui";
import { withAlpha } from "@/utils/color";

// Identité de Baskul : voir docs/identite.md

const FONT_FAMILY =
  'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif';
const FONT_FAMILY_MONO =
  'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace';

const COMMON: GlobalThemeOverrides["common"] = {
  fontFamily: FONT_FAMILY,
  fontFamilyMono: FONT_FAMILY_MONO,
  fontWeightStrong: "600",
  borderRadius: "10px",
  borderRadiusSmall: "6px",
  scrollbarWidth: "6px",
  scrollbarHeight: "6px",
  scrollbarBorderRadius: "6px",
};

interface Shades {
  color: string;
  hover: string;
  pressed: string;
}

type Family = "primary" | "info" | "success" | "warning" | "error";

// Une famille de couleurs Naive UI depuis trois teintes : Suppl, l'accompagnement, reprend le survol
function family(name: Family, { color, hover, pressed }: Shades): GlobalThemeOverrides["common"] {
  return {
    [`${name}Color`]: color,
    [`${name}ColorHover`]: hover,
    [`${name}ColorPressed`]: pressed,
    [`${name}ColorSuppl`]: hover,
  };
}

// Accent violet, teinte OKLCH 285° : choisi hors des couleurs d'état (docs/identite.md)
const ACCENT_LIGHT: Shades = { color: "#745af4", hover: "#8472ff", pressed: "#6445df" };
const ACCENT_DARK: Shades = { color: "#968cff", hover: "#a6a1ff", pressed: "#8477f1" };

const LIGHT: GlobalThemeOverrides["common"] = {
  ...family("primary", ACCENT_LIGHT),
  ...family("info", ACCENT_LIGHT),
  ...family("success", { color: "#16a34a", hover: "#22c55e", pressed: "#15803d" }),
  ...family("warning", { color: "#d97706", hover: "#f59e0b", pressed: "#b45309" }),
  ...family("error", { color: "#dc2626", hover: "#ef4444", pressed: "#b91c1c" }),
  bodyColor: "#f4f6fb",
  cardColor: "#ffffff",
  modalColor: "#ffffff",
  popoverColor: "#ffffff",
  tableColor: "#ffffff",
  textColor1: "#151a2d",
  textColor2: "#2a3045",
  textColor3: "#6b7290",
  borderColor: "#e2e6f0",
  dividerColor: "#eceff5",
  tagColor: "#eef0f8",
  actionColor: "#f7f8fc",
  tabColor: "#f1f3f9",
  hoverColor: withAlpha(ACCENT_LIGHT.color, 0.07),
  scrollbarColor: "rgba(21, 26, 45, 0.22)",
  scrollbarColorHover: "rgba(21, 26, 45, 0.36)",
};

const DARK: GlobalThemeOverrides["common"] = {
  ...family("primary", ACCENT_DARK),
  ...family("info", ACCENT_DARK),
  ...family("success", { color: "#4ade80", hover: "#6ee79a", pressed: "#3cc96f" }),
  ...family("warning", { color: "#fbbf24", hover: "#fcd34d", pressed: "#f59e0b" }),
  ...family("error", { color: "#f87171", hover: "#fca5a5", pressed: "#ef4444" }),
  bodyColor: "#0d1020",
  cardColor: "#151a2e",
  modalColor: "#1a2036",
  popoverColor: "#1e2540",
  tableColor: "#151a2e",
  textColor1: "rgba(236, 239, 255, 0.94)",
  textColor2: "rgba(236, 239, 255, 0.82)",
  textColor3: "rgba(236, 239, 255, 0.54)",
  borderColor: "rgba(160, 170, 255, 0.16)",
  dividerColor: "rgba(160, 170, 255, 0.09)",
  tagColor: withAlpha(ACCENT_DARK.color, 0.12),
  actionColor: "rgba(255, 255, 255, 0.04)",
  tabColor: "rgba(255, 255, 255, 0.05)",
  inputColor: "rgba(255, 255, 255, 0.06)",
  hoverColor: withAlpha(ACCENT_DARK.color, 0.1),
  scrollbarColor: "rgba(160, 170, 255, 0.24)",
  scrollbarColorHover: "rgba(160, 170, 255, 0.4)",
};

/** Rayon des panneaux flottants : cartes et cadre de la fenêtre (AppLayout.vue) */
export const PANEL_RADIUS = "16px";

// Le fond de la fenêtre apparaît entre les panneaux ; la couleur du cadre est posée par AppLayout.vue
const COMPONENTS: GlobalThemeOverrides = {
  Layout: {
    color: "transparent",
  },
  Card: {
    borderRadius: PANEL_RADIUS,
  },
  Button: {
    borderRadiusTiny: "6px",
    borderRadiusSmall: "8px",
    borderRadiusMedium: "8px",
    borderRadiusLarge: "10px",
    fontWeight: "500",
  },
  Menu: {
    borderRadius: "10px",
    itemHeight: "40px",
  },
  Tag: {
    borderRadius: "6px",
  },
};

export function themeOverrides(isDark: boolean): GlobalThemeOverrides {
  return {
    ...COMPONENTS,
    common: { ...COMMON, ...(isDark ? DARK : LIGHT) },
  };
}
