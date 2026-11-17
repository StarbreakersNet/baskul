import { defineStore } from "pinia";
import { ref } from "vue";
import { useAppMessage } from "@/composables/useAppMessage";
import { useLoader } from "@/composables/useLoader";
import { useStatusStore } from "@/stores/status";
import { daemonClient, errorMessage } from "@/utils/daemonClient";
import { i18n } from "@/utils/locale";
import type { ProfilesInfo } from "@/types/Profiles";

export const useProfilesStore = defineStore("profiles", () => {
  const message = useAppMessage();

  const names = ref<string[]>([]);
  const active = ref<string | null>(null);
  const isLoaded = ref(false);
  const applying = useLoader();

  function show(info: ProfilesInfo) {
    names.value = info.profiles;
    active.value = info.active;
    isLoaded.value = true;
  }

  async function refresh() {
    try {
      show(await daemonClient.profiles());
    } catch (error) {
      console.error(error);
    }
  }

  async function save(name: string) {
    try {
      show(await daemonClient.saveProfile(name));
      message.success(i18n.global.t("profiles.saved", { name }));
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  async function remove(name: string) {
    try {
      show(await daemonClient.deleteProfile(name));
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  /** Baskul redémarre avec les réglages du profil */
  async function apply(name: string) {
    try {
      const result = await applying.run(() => daemonClient.applyProfile(name));
      if (!result.restart || (await useStatusStore().waitForRestart())) {
        message.success(i18n.global.t("profiles.applied", { name }));
      }
      await refresh();
    } catch (error) {
      message.error(errorMessage(error));
    }
  }

  return { names, active, isLoaded, applying, refresh, save, remove, apply };
});
