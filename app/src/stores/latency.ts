import { defineStore } from "pinia";
import { ref } from "vue";
import { daemonClient } from "@/utils/daemonClient";
import type { LatencyInfo } from "@/types/Latency";

export const useLatencyStore = defineStore("latency", () => {
  const peers = ref<LatencyInfo["peers"]>([]);

  async function refresh() {
    try {
      peers.value = (await daemonClient.readLatency()).peers;
    } catch (error) {
      console.error(error);
    }
  }

  return { peers, refresh };
});
