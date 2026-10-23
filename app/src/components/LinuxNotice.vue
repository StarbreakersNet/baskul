<script setup lang="ts">
// Rappel : la version Linux ne partage pas le clavier et la souris.
import { inject } from "vue";
import { NAlert } from "naive-ui";
import { osKey } from "../keys";

const os = inject(osKey);
</script>

<template>
  <n-alert v-if="os === 'linux'" type="info" :bordered="false" style="margin: 8px 0 4px">
    {{ $t("settings.linuxNotice") }}
  </n-alert>
</template>
