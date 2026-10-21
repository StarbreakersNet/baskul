<script setup lang="ts">
// État de la connexion, mis à jour en direct.
import { computed } from "vue";
import { NTag } from "naive-ui";
import type { DaemonStatus } from "../types";

const props = defineProps<{ status: DaemonStatus | null; restarting?: boolean }>();

const view = computed(() => {
  if (props.restarting) return { type: "info", text: "Redémarrage…" } as const;
  const s = props.status;
  if (!s) return { type: "default", text: "Bascule n'est pas lancé" } as const;
  const text = s.summary || "Démarrage…";
  if (s.state === "connected") return { type: "success", text } as const;
  if (s.state === "error") return { type: "error", text } as const;
  return { type: "warning", text } as const;
});
</script>

<template>
  <n-tag :type="view.type" round :bordered="false" class="status">
    <span class="dot" />
    {{ view.text }}
  </n-tag>
</template>

<style scoped>
.status {
  max-width: 100%;
}
.dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  margin-right: 6px;
  border-radius: 50%;
  background: currentColor;
}
</style>
