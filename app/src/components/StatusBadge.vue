<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HTag from "@/components/c/hTag.vue";
import type { TagProps } from "naive-ui";
import type { DaemonStatus } from "@/types/Daemon";

const { status, isRestarting = false } = defineProps<{
  status: DaemonStatus | null;
  isRestarting?: boolean;
}>();

const { t } = useI18n();

const STATE_TYPES: Record<DaemonStatus["state"], TagProps["type"]> = {
  connected: "success",
  error: "error",
  waiting: "warning",
};

const badgeType = computed<TagProps["type"]>(() => {
  if (isRestarting) {
    return "info";
  }
  return status ? STATE_TYPES[status.state] : "default";
});

// Le résumé vient de Baskul, déjà traduit
const badgeText = computed(() => {
  if (isRestarting) {
    return t("status.restarting");
  }
  return status ? status.summary || t("status.starting") : t("status.notRunning");
});
</script>

<template>
  <h-tag :bordered="false" class="status-badge" round :type="badgeType">
    <span class="status-badge-dot" />
    {{ badgeText }}
  </h-tag>
</template>

<style lang="scss" scoped>
.status-badge {
  max-width: 100%;
}

.status-badge-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  margin-right: 6px;
  border-radius: 50%;
  background: currentColor;
}
</style>
