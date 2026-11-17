<script setup lang="ts">
import { inject } from "vue";
import HAlert from "@/components/c/hAlert.vue";
import { OS_KEY } from "@/utils/injectionKeys";

const os = inject(OS_KEY);
</script>

<template>
  <h-alert v-if="os === 'linux'" :bordered="false" class="linux-notice" type="info">
    {{ $t("settings.linuxNotice") }}
  </h-alert>
</template>

<style lang="scss" scoped>
.linux-notice {
  margin: 8px 0 4px;
}
</style>
