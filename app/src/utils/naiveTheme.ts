import type { GlobalThemeOverrides } from "naive-ui";

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

const LIGHT: GlobalThemeOverrides["common"] = {
  primaryColor: "#4c5fef",
  primaryColorHover: "#6878f5",
  primaryColorPressed: "#3a4ad8",
  primaryColorSuppl: "#6878f5",
  infoColor: "#4c5fef",
  infoColorHover: "#6878f5",
  infoColorPressed: "#3a4ad8",
  infoColorSuppl: "#6878f5",
  successColor: "#16a34a",
  successColorHover: "#22c55e",
  successColorPressed: "#15803d",
  successColorSuppl: "#22c55e",
  warningColor: "#d97706",
  warningColorHover: "#f59e0b",
  warningColorPressed: "#b45309",
  warningColorSuppl: "#f59e0b",
  errorColor: "#dc2626",
  errorColorHover: "#ef4444",
  errorColorPressed: "#b91c1c",
  errorColorSuppl: "#ef4444",
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
  hoverColor: "rgba(76, 95, 239, 0.07)",
  scrollbarColor: "rgba(21, 26, 45, 0.22)",
  scrollbarColorHover: "rgba(21, 26, 45, 0.36)",
};

const DARK: GlobalThemeOverrides["common"] = {
  primaryColor: "#8c9bff",
  primaryColorHover: "#a5b1ff",
  primaryColorPressed: "#7383f0",
  primaryColorSuppl: "#a5b1ff",
  infoColor: "#8c9bff",
  infoColorHover: "#a5b1ff",
  infoColorPressed: "#7383f0",
  infoColorSuppl: "#a5b1ff",
  successColor: "#4ade80",
  successColorHover: "#6ee79a",
  successColorPressed: "#3cc96f",
  successColorSuppl: "#6ee79a",
  warningColor: "#fbbf24",
  warningColorHover: "#fcd34d",
  warningColorPressed: "#f59e0b",
  warningColorSuppl: "#fcd34d",
  errorColor: "#f87171",
  errorColorHover: "#fca5a5",
  errorColorPressed: "#ef4444",
  errorColorSuppl: "#fca5a5",
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
  tagColor: "rgba(140, 155, 255, 0.12)",
  actionColor: "rgba(255, 255, 255, 0.04)",
  tabColor: "rgba(255, 255, 255, 0.05)",
  inputColor: "rgba(255, 255, 255, 0.06)",
  hoverColor: "rgba(140, 155, 255, 0.1)",
  scrollbarColor: "rgba(160, 170, 255, 0.24)",
  scrollbarColorHover: "rgba(160, 170, 255, 0.4)",
};

// Le fond de la fenêtre (dégradés de la marque, assets/brand.scss) doit transparaître sous le cadre
const COMPONENTS: GlobalThemeOverrides = {
  Layout: {
    color: "transparent",
    siderColor: "transparent",
    headerColor: "transparent",
    footerColor: "transparent",
  },
  Card: {
    borderRadius: "14px",
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
