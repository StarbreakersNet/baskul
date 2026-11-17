<script setup lang="ts">
import { nextTick, onActivated, onBeforeUnmount, onDeactivated, ref, useTemplateRef } from "vue";
import HButton from "@/components/c/hButton.vue";
import HCard from "@/components/c/hCard.vue";
import HFlex from "@/components/c/hFlex.vue";
import HLog from "@/components/c/hLog.vue";
import HSwitch from "@/components/c/hSwitch.vue";
import HText from "@/components/c/hText.vue";
import { useLogStore } from "@/stores/log";
import type { HLogInst } from "@/types/components/c/hLog";

const REFRESH_MS = 2000;

const logStore = useLogStore();

const isFollowing = ref(true);
const logRef = useTemplateRef<HLogInst>("logRef");

let refreshTimer: ReturnType<typeof setInterval> | undefined;

async function refresh() {
  if ((await logStore.refresh()) && isFollowing.value) {
    await nextTick();
    logRef.value?.scrollToBottom();
  }
}

function startRefreshing() {
  clearInterval(refreshTimer);
  void refresh();
  refreshTimer = setInterval(refresh, REFRESH_MS);
}

function stopRefreshing() {
  clearInterval(refreshTimer);
}

onActivated(startRefreshing);
onDeactivated(stopRefreshing);
onBeforeUnmount(stopRefreshing);
</script>

<template>
  <h-card size="small" :title="$t('log.title')">
    <template #header-extra>
      <h-flex align="center" :size="12">
        <h-flex align="center" :size="6">
          <h-switch v-model:value="isFollowing" size="small" />
          <h-text depth="3">{{ $t("log.follow") }}</h-text>
        </h-flex>
        <h-button size="small" @click="logStore.openFile()">{{ $t("common.openFile") }}</h-button>
      </h-flex>
    </template>
    <h-log ref="logRef" :loading="!logStore.isLoaded" :log="logStore.text" :rows="26" trim />
    <template #footer>
      <h-text class="mono break-all log-path" depth="3">{{ logStore.path }}</h-text>
    </template>
  </h-card>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.log-path {
  font-size: variables.$font-size-small;
}
</style>
