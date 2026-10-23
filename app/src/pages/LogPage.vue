<script setup lang="ts">
// Journal de Bascule : les dernières lignes, rafraîchies en direct.
import { nextTick, onActivated, onDeactivated, ref } from "vue";
import { NButton, NCard, NFlex, NLog, NSwitch, NText } from "naive-ui";
import { useActions } from "../composables/useActions";
import { daemon } from "../daemon";

const REFRESH_MS = 2000;

const actions = useActions();
const text = ref("");
const path = ref<string | null>(null);
const loading = ref(true);
/** Suivre la fin du journal. */
const follow = ref(true);
const log = ref<InstanceType<typeof NLog>>();

async function refresh() {
  try {
    const r = await daemon.readLog();
    path.value = r.path;
    if (r.text !== text.value) {
      text.value = r.text;
      if (follow.value) {
        await nextTick();
        log.value?.scrollTo({ position: "bottom", silent: true });
      }
    }
  } catch {
    // Bascule redémarre : nouvel essai au prochain tour.
  } finally {
    loading.value = false;
  }
}

// Rafraîchi seulement quand la page est affichée (elle reste en mémoire).
let timer: ReturnType<typeof setInterval> | undefined;
onActivated(() => {
  void refresh();
  timer = setInterval(refresh, REFRESH_MS);
});
onDeactivated(() => clearInterval(timer));
</script>

<template>
  <n-card size="small" :title="$t('log.title')">
    <template #header-extra>
      <n-flex align="center" :size="12">
        <n-flex align="center" :size="6">
          <n-switch v-model:value="follow" size="small" />
          <n-text depth="3">{{ $t("log.follow") }}</n-text>
        </n-flex>
        <n-button size="small" @click="actions.openLogFile">{{ $t("common.openFile") }}</n-button>
      </n-flex>
    </template>
    <n-log ref="log" :log="text" :rows="26" :loading="loading" trim />
    <template #footer>
      <n-text depth="3" class="mono" style="font-size: 12.5px; word-break: break-all">{{ path }}</n-text>
    </template>
  </n-card>
</template>
