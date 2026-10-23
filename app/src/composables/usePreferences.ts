// Thème et langue de la fenêtre : ceux des réglages (aperçu immédiat avant
// même l'enregistrement), mémorisés pour s'afficher dès l'ouverture suivante.
import { computed, watch } from "vue";
import { useOsTheme } from "naive-ui";
import { i18n, systemLocale, type Locale } from "../i18n";
import type { BasculeConfig } from "../types";
import { useSettings } from "./useSettings";

type Ui = BasculeConfig["ui"];
const STORAGE_KEY = "bascule.ui";

function cached(): Partial<Ui> {
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") as Partial<Ui>;
  } catch {
    return {};
  }
}

export function usePreferences() {
  const { config } = useSettings();
  const osTheme = useOsTheme();
  const ui = computed<Partial<Ui>>(() => config.value?.ui ?? cached());

  const dark = computed(() => {
    const theme = ui.value.theme ?? "system";
    return theme === "system" ? osTheme.value === "dark" : theme === "dark";
  });
  const locale = computed<Locale>(() => {
    const language = ui.value.language ?? "system";
    return language === "system" ? systemLocale() : language;
  });

  watch(
    locale,
    (l) => {
      i18n.global.locale.value = l;
      document.documentElement.lang = l;
    },
    { immediate: true },
  );
  watch(
    () => config.value?.ui,
    (u) => {
      if (!u) return;
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme: u.theme, language: u.language }));
      } catch {
        // Stockage indisponible : préférences relues à l'ouverture suivante.
      }
    },
    { deep: true, immediate: true },
  );

  return { dark, locale };
}
