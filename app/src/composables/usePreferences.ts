import { computed, watch } from "vue";
import { useOsTheme } from "naive-ui";
import { useSettingsStore } from "@/stores/settings";
import { i18n, systemLocale, type Locale } from "@/utils/locale";
import type { BaskulConfig } from "@/types/Config";

type UiConfig = BaskulConfig["ui"];

// Mémorisés pour que la fenêtre s'ouvre dans le bon thème avant la réponse de Baskul
const STORAGE_KEY = "baskul.ui";

function cachedUi(): Partial<UiConfig> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") as Partial<UiConfig>;
  } catch {
    return {};
  }
}

function cacheUi(ui: UiConfig) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: ui.theme, language: ui.language }));
  } catch {
    // Stockage indisponible : la fenêtre s'ouvrira avec le thème du système
  }
}

/** Thème et langue des réglages, appliqués avant même l'enregistrement */
export function usePreferences() {
  const settings = useSettingsStore();
  const osTheme = useOsTheme();

  const ui = computed<Partial<UiConfig>>(() => settings.config?.ui ?? cachedUi());

  const isDark = computed(() => {
    const theme = ui.value.theme ?? "system";
    return theme === "system" ? osTheme.value === "dark" : theme === "dark";
  });

  const locale = computed<Locale>(() => {
    const language = ui.value.language ?? "system";
    return language === "system" ? systemLocale() : language;
  });

  // Pour les styles qui ne viennent pas de Naive UI (assets/brand.scss)
  watch(
    isDark,
    value => {
      document.documentElement.dataset.theme = value ? "dark" : "light";
    },
    { immediate: true }
  );

  watch(
    locale,
    value => {
      i18n.global.locale.value = value;
      document.documentElement.lang = value;
    },
    { immediate: true }
  );

  watch(
    () => settings.config?.ui,
    value => {
      if (value) {
        cacheUi(value);
      }
    },
    { deep: true, immediate: true }
  );

  return { isDark, locale };
}
