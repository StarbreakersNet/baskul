<script setup lang="ts">
import { computed, onActivated, onBeforeUnmount, onDeactivated } from "vue";
import { useThemeVars } from "naive-ui";
import HCard from "@/components/c/hCard.vue";
import HFlex from "@/components/c/hFlex.vue";
import HTag from "@/components/c/hTag.vue";
import HText from "@/components/c/hText.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import { useLatencyStore } from "@/stores/latency";
import { formatMs, latencyStats, sparklinePoints, type LinkQuality } from "@/utils/latency";

const REFRESH_MS = 2000;
const CHART_WIDTH = 180;
const CHART_HEIGHT = 32;
const QUALITY_TAGS: Record<LinkQuality, "success" | "warning" | "error"> = {
  good: "success",
  fair: "warning",
  poor: "error",
};

const latency = useLatencyStore();
const themeVars = useThemeVars();

const rows = computed(() =>
  latency.peers.map(peer => ({
    name: peer.name,
    stats: latencyStats(peer.samples),
    points: sparklinePoints(peer.samples, CHART_WIDTH, CHART_HEIGHT),
  }))
);
const lineColor = computed(() => themeVars.value.primaryColor);

let refreshTimer: ReturnType<typeof setInterval> | undefined;

function startRefreshing() {
  clearInterval(refreshTimer);
  void latency.refresh();
  refreshTimer = setInterval(latency.refresh, REFRESH_MS);
}

function stopRefreshing() {
  clearInterval(refreshTimer);
}

onActivated(startRefreshing);
onDeactivated(stopRefreshing);
onBeforeUnmount(stopRefreshing);
</script>

<template>
  <expand-transition>
    <div v-if="rows.length" class="network-card">
      <h-card size="small" :title="$t('network.title')">
        <expand-transition group>
          <div v-for="row in rows" :key="row.name" class="network-row">
            <h-flex align="center" justify="space-between" :size="16">
              <h-flex :size="2" vertical>
                <h-text strong>{{ row.name }}</h-text>
                <h-text v-if="row.stats" class="small-text" depth="3">
                  {{
                    $t("network.summary", {
                      average: formatMs(row.stats.average),
                      max: formatMs(row.stats.max),
                    })
                  }}
                </h-text>
              </h-flex>
              <h-flex align="center" :size="12" :wrap="false">
                <svg
                  aria-hidden="true"
                  class="network-chart"
                  :height="CHART_HEIGHT"
                  :viewBox="`0 0 ${CHART_WIDTH} ${CHART_HEIGHT}`"
                  :width="CHART_WIDTH">
                  <polyline
                    fill="none"
                    :points="row.points"
                    :stroke="lineColor"
                    stroke-linejoin="round"
                    stroke-width="1.5" />
                </svg>
                <transition mode="out-in" name="fade">
                  <h-tag
                    v-if="row.stats"
                    :key="row.stats.quality"
                    :bordered="false"
                    class="network-value"
                    round
                    :type="QUALITY_TAGS[row.stats.quality]">
                    {{ $t("network.ms", { ms: formatMs(row.stats.last) }) }}
                  </h-tag>
                </transition>
              </h-flex>
            </h-flex>
          </div>
        </expand-transition>
        <template #footer>
          <h-text class="small-text" depth="3">{{ $t("network.help") }}</h-text>
        </template>
      </h-card>
    </div>
  </expand-transition>
</template>

<style lang="scss" scoped>
// Écart en padding : il grandit avec la carte
.network-card {
  padding-top: 16px;
}

.network-row {
  padding: 6px 0;
}

.network-chart {
  flex: none;
  overflow: visible;
}

.network-value {
  min-width: 64px;
  justify-content: center;
  font-variant-numeric: tabular-nums;
}
</style>
