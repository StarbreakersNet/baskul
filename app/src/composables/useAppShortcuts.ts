import { onBeforeUnmount, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useSettingsStore } from "@/stores/settings";

// Raccourcis de la fenêtre (Ctrl, ou Cmd sur macOS) : chiffres pour les pages, S pour enregistrer
const PAGE_KEYS: Record<string, string> = {
  "1": "home",
  "2": "settings",
  "3": "log",
  "4": "about",
};

export function useAppShortcuts() {
  const router = useRouter();
  const settings = useSettingsStore();

  function handleKeyDown(event: KeyboardEvent) {
    const hasModifier = (event.ctrlKey || event.metaKey) && !event.altKey && !event.shiftKey;
    const page = PAGE_KEYS[event.key];
    if (hasModifier && page) {
      event.preventDefault();
      router.push({ name: page });
    } else if (hasModifier && event.key.toLowerCase() === "s") {
      event.preventDefault();
      saveIfNeeded();
    }
  }

  function saveIfNeeded() {
    if (settings.isDirty && !settings.saving.loading) {
      void settings.save();
    }
  }

  onMounted(() => window.addEventListener("keydown", handleKeyDown));
  onBeforeUnmount(() => window.removeEventListener("keydown", handleKeyDown));
}
