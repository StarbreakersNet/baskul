import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import type { UpdateInfo } from "@/types/Update";

// La recherche continue dans Baskul après la réponse : son état arrive ensuite avec celui de Baskul
const CHECK_FEEDBACK_MS = 1500;

export const useUpdateStore = defineStore("update", () => {
  const message = useAppMessage();
  const statusStore = useStatusStore();

  const lastCheck = ref<UpdateInfo | null>(null);
  const checking = useLoader();

  const info = computed<UpdateInfo | null>(() => statusStore.status?.update ?? lastCheck.value);

  async function check() {
    await checking.run(async () => {
      try {
        lastCheck.value = await daemonClient.checkUpdate();
      } catch (error) {
        message.error(errorMessage(error));
      }
      await new Promise(resolve => setTimeout(resolve, CHECK_FEEDBACK_MS));
    });
  }

  /** Renvoie `true` si l'installation a démarré : Baskul redémarre ensuite */
  async function install() {
    let started = false;
    try {
      await daemonClient.installUpdate();
      started = true;
    } catch (error) {
      message.error(errorMessage(error));
    }
    return started;
  }

  return { info, checking, check, install };
});
