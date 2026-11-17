import { defineStore } from "pinia";
import { ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { daemonClient, errorMessage } from "@/utils/daemonClient";

export const useLogStore = defineStore("log", () => {
  const message = useAppMessage();

  const text = ref("");
  const path = ref<string | null>(null);
  const isLoaded = ref(false);

  /** Renvoie `true` si le texte a changé */
  async function refresh() {
    let changed = false;
    try {
      const tail = await daemonClient.readLog();
      path.value = tail.path;
      changed = tail.text !== text.value;
      text.value = tail.text;
    } catch {
      // Baskul redémarre : le prochain rafraîchissement relira le journal
    }
    isLoaded.value = true;
    return changed;
  }

  async function openFile() {
    try {
      await daemonClient.openLogFile();
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return { text, path, isLoaded, refresh, openFile };
});
