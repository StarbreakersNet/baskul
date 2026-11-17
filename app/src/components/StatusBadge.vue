<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import HTag from "@/components/c/hTag.vue";
import type { TagProps } from "naive-ui";
import type { DaemonStatus } from "@/types/Daemon";
import { PULSE_SECONDS } from "@/utils/backgroundPatterns";

const { status, isRestarting = false } = defineProps<{
  status: DaemonStatus | null;
  isRestarting?: boolean;
}>();

const { t } = useI18n();

const PULSE_DURATION = `${PULSE_SECONDS}s`;

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

const isPulsing = computed(
  () => isRestarting || status?.state === "connected" || status?.state === "waiting"
);

// Le résumé vient de Baskul, déjà traduit
const badgeText = computed(() => {
  if (isRestarting) {
    return t("status.restarting");
  }
  return status ? status.summary || t("status.starting") : t("status.notRunning");
});
</script>

<template>
  <h-tag :bordered="false" class="status-badge" round :title="badgeText" :type="badgeType">
    <span :class="['status-badge-dot', { 'status-badge-dot--pulse': isPulsing }]" />
    <span class="status-badge-text">{{ badgeText }}</span>
  </h-tag>
</template>

<style lang="scss" scoped>
$text-max-width: 260px;

.status-badge {
  max-width: 100%;
}

.status-badge-dot {
  position: relative;
  display: inline-block;
  flex: none;
  width: 8px;
  height: 8px;
  margin-right: 6px;
  border-radius: 50%;
  background: currentColor;
  vertical-align: middle;
}

.status-badge-dot--pulse::after {
  content: "";
  position: absolute;
  inset: 0;
  border-radius: 50%;
  background: currentColor;
  animation: status-badge-pulse v-bind(PULSE_DURATION) ease-out infinite;
}

.status-badge-text {
  display: inline-block;
  max-width: $text-max-width;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: middle;
}

@keyframes status-badge-pulse {
  0% {
    transform: scale(1);
    opacity: 0.55;
  }

  70%,
  100% {
    transform: scale(3);
    opacity: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .status-badge-dot--pulse::after {
    animation: none;
  }
}
</style>
