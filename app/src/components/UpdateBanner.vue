<script setup lang="ts">
// Nouvelle version téléchargée : un redémarrage l'installe.
import { NAlert, NButton, NFlex, NText } from "naive-ui";
import { useUpdate } from "../composables/useUpdate";

const { info, install } = useUpdate();
</script>

<template>
  <n-alert
    v-if="info && (info.state === 'ready' || info.state === 'installing')"
    type="success"
    :title="$t('update.readyTitle', { version: info.version })"
    style="margin-bottom: 16px"
  >
    <n-flex align="center" justify="space-between" :size="12">
      <n-text>{{ $t("update.readyText") }}</n-text>
      <n-button type="primary" size="small" :loading="info.state === 'installing'" @click="install">
        {{ $t("update.restart") }}
      </n-button>
    </n-flex>
  </n-alert>
</template>
