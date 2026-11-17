<script setup lang="ts">
import { inject } from "vue";
import HAlert from "@/components/c/hAlert.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import { OS_KEY } from "@/utils/injectionKeys";

const os = inject(OS_KEY);
</script>

<template>
  <expand-transition>
    <div v-if="os === 'linux'" class="linux-notice">
      <h-alert :bordered="false" type="info">{{ $t("settings.linuxNotice") }}</h-alert>
    </div>
  </expand-transition>
</template>

<style lang="scss" scoped>
.linux-notice {
  padding: 8px 0 4px;
}
</style>
