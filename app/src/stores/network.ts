import { defineStore } from "pinia";
import { ref } from "vue";
import { daemonClient } from "@/utils/daemonClient";
import type { NetworkInfo } from "@/types/Network";

export const useNetworkStore = defineStore("network", () => {
  const network = ref<NetworkInfo | null>(null);

  async function refresh() {
    try {
      network.value = await daemonClient.network();
    } catch {
      // Baskul redémarre : on garde la dernière réponse
    }
  }

  return { network, refresh };
});
