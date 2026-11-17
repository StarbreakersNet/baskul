import { defineStore } from "pinia";
import { ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import type { ActivityEvent } from "@/types/Activity";

export const useActivityStore = defineStore("activity", () => {
  const message = useAppMessage();

  /** Du plus récent au plus ancien */
  const events = ref<ActivityEvent[]>([]);
  const isLoaded = ref(false);

  let syncedId: number | null = null;

  /** `lastId` : identifiant du dernier événement, donné par l'état de Baskul */
  async function sync(lastId: number) {
    if (lastId !== syncedId) {
      syncedId = lastId;
      try {
        const { events: list } = await daemonClient.readActivity();
        events.value = [...list].reverse();
      } catch (error) {
        console.error(error);
      }
      isLoaded.value = true;
    }
  }

  async function clear() {
    try {
      await daemonClient.clearActivity();
      events.value = [];
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return { events, isLoaded, sync, clear };
});
