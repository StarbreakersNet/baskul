import type { GlobalThemeOverrides } from "naive-ui";

// Identité de Baskul, style Liquid Glass : voir docs/identite.md

const FONT_FAMILY =
  'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", sans-serif';
const FONT_FAMILY_MONO =
  'ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace';
const PILL = "999px";

const COMMON: GlobalThemeOverrides["common"] = {
  fontFamily: FONT_FAMILY,
  fontFamilyMono: FONT_FAMILY_MONO,
  fontWeightStrong: "600",
  borderRadius: "12px",
  borderRadiusSmall: "8px",
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
  bodyColor: "#eef1fa",
  cardColor: "rgba(255, 255, 255, 0.52)",
  modalColor: "rgba(255, 255, 255, 0.8)",
  popoverColor: "rgba(255, 255, 255, 0.82)",
  tableColor: "rgba(255, 255, 255, 0.4)",
  inputColor: "rgba(255, 255, 255, 0.62)",
  actionColor: "rgba(255, 255, 255, 0.38)",
  tabColor: "rgba(255, 255, 255, 0.45)",
  textColor1: "#151a2d",
  textColor2: "#2a3045",
  textColor3: "#5f6684",
  borderColor: "rgba(21, 26, 45, 0.12)",
  dividerColor: "rgba(21, 26, 45, 0.07)",
  tagColor: "rgba(255, 255, 255, 0.55)",
  hoverColor: "rgba(76, 95, 239, 0.08)",
  scrollbarColor: "rgba(21, 26, 45, 0.22)",
  scrollbarColorHover: "rgba(21, 26, 45, 0.36)",
  boxShadow2: "0 1px 0 rgba(255, 255, 255, 0.7) inset, 0 12px 32px -8px rgba(30, 40, 110, 0.22)",
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
  bodyColor: "#0a0d1c",
  cardColor: "rgba(34, 40, 72, 0.42)",
  modalColor: "rgba(26, 31, 56, 0.86)",
  popoverColor: "rgba(32, 38, 68, 0.88)",
  tableColor: "rgba(255, 255, 255, 0.03)",
  inputColor: "rgba(255, 255, 255, 0.07)",
  actionColor: "rgba(255, 255, 255, 0.04)",
  tabColor: "rgba(255, 255, 255, 0.06)",
  textColor1: "rgba(240, 242, 255, 0.95)",
  textColor2: "rgba(240, 242, 255, 0.82)",
  textColor3: "rgba(240, 242, 255, 0.56)",
  borderColor: "rgba(200, 210, 255, 0.14)",
  dividerColor: "rgba(200, 210, 255, 0.08)",
  tagColor: "rgba(255, 255, 255, 0.08)",
  hoverColor: "rgba(140, 155, 255, 0.12)",
  scrollbarColor: "rgba(200, 210, 255, 0.24)",
  scrollbarColorHover: "rgba(200, 210, 255, 0.4)",
  boxShadow2: "0 1px 0 rgba(255, 255, 255, 0.1) inset, 0 16px 40px -8px rgba(0, 0, 0, 0.55)",
};

function components(isDark: boolean): GlobalThemeOverrides {
  const glassBorder = isDark ? "rgba(255, 255, 255, 0.1)" : "rgba(255, 255, 255, 0.7)";
  const bar = isDark ? "rgba(20, 24, 46, 0.5)" : "rgba(255, 255, 255, 0.5)";
  return {
    Layout: {
      color: "transparent",
      siderColor: bar,
      headerColor: bar,
      footerColor: bar,
      siderBorderColor: glassBorder,
      headerBorderColor: glassBorder,
      footerBorderColor: glassBorder,
    },
    Card: {
      borderRadius: "22px",
      borderColor: glassBorder,
      colorEmbedded: isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(255, 255, 255, 0.42)",
      colorEmbeddedModal: isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(255, 255, 255, 0.42)",
    },
    Button: {
      borderRadiusTiny: PILL,
      borderRadiusSmall: PILL,
      borderRadiusMedium: PILL,
      borderRadiusLarge: PILL,
      fontWeight: "500",
    },
    Input: {
      borderRadius: "12px",
    },
    Menu: {
      borderRadius: "14px",
      itemHeight: "42px",
    },
    Tabs: {
      tabBorderRadius: PILL,
      colorSegment: isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(255, 255, 255, 0.45)",
      tabColorSegment: isDark ? "rgba(255, 255, 255, 0.12)" : "rgba(255, 255, 255, 0.9)",
    },
    Tag: {
      borderRadius: PILL,
    },
    Alert: {
      borderRadius: "18px",
    },
    Dialog: {
      borderRadius: "24px",
    },
    Popover: {
      borderRadius: "16px",
    },
  };
}

export function themeOverrides(isDark: boolean): GlobalThemeOverrides {
  return {
    ...components(isDark),
    common: { ...COMMON, ...(isDark ? DARK : LIGHT) },
  };
}
