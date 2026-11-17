<script setup lang="ts">
import { computed } from "vue";
import HAlert from "@/components/c/hAlert.vue";
import HButton from "@/components/c/hButton.vue";
import HFlex from "@/components/c/hFlex.vue";
import HText from "@/components/c/hText.vue";
import { useUpdateInstall } from "@/composables/useUpdateInstall";
import { useUpdateStore } from "@/stores/update";

const update = useUpdateStore();
const { install } = useUpdateInstall();

const readyVersion = computed(() => {
  const info = update.info;
  return info?.state === "ready" || info?.state === "installing" ? info.version : null;
});
const isInstalling = computed(() => update.info?.state === "installing");
</script>

<template>
  <h-alert
    v-if="readyVersion"
    class="update-banner"
    :title="$t('update.readyTitle', { version: readyVersion })"
    type="success">
    <h-flex align="center" justify="space-between" :size="12">
      <h-text>{{ $t("update.readyText") }}</h-text>
      <h-button :loading="isInstalling" size="small" type="primary" @click="install()">
        {{ $t("update.restart") }}
      </h-button>
    </h-flex>
  </h-alert>
</template>

<style lang="scss" scoped>
.update-banner {
  margin-bottom: 16px;
}
</style>
