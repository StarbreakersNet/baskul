// Traductions avec vue-i18n. Les textes sont dans locales/*.json, à la racine
// du dépôt : section « app » pour cette fenêtre, section « daemon » pour les
// menus de Bascule.
import { createI18n } from "vue-i18n";
import en from "../../locales/en.json";
import fr from "../../locales/fr.json";

export type Locale = "fr" | "en";

/** Langue du système : français ou, à défaut, anglais. */
export function systemLocale(): Locale {
  return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

export const i18n = createI18n({
  legacy: false,
  locale: systemLocale(),
  fallbackLocale: "en",
  messages: { fr: fr.app, en: en.app },
});
