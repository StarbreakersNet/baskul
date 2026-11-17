import { createI18n } from "vue-i18n";
import en from "@locales/en.json";
import fr from "@locales/fr.json";

export type Locale = "fr" | "en";

export function systemLocale(): Locale {
  return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

// Section "app" des traductions : la section "daemon" sert aux menus de Baskul (Rust)
export const i18n = createI18n({
  legacy: false,
  locale: systemLocale(),
  fallbackLocale: "en",
  messages: { fr: fr.app, en: en.app },
});
