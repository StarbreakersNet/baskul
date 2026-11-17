<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import HFlex from "@/components/c/hFlex.vue";
import HText from "@/components/c/hText.vue";
import logo from "@/assets/logo.svg";

const themeVars = useThemeVars();

const background = computed(() => themeVars.value.bodyColor);
</script>

<template>
  <h-flex align="center" class="app-splash" justify="center" vertical>
    <img alt="" class="app-splash-logo" :src="logo" />
    <h-text class="brand-wordmark app-splash-name">Baskul</h-text>
    <h-text depth="3">{{ $t("splash.connecting") }}</h-text>
  </h-flex>
</template>

<style lang="scss" scoped>
.app-splash {
  position: fixed;
  inset: 0;
  z-index: 20;
  background: v-bind(background);
}

.app-splash-logo {
  width: 96px;
  height: 96px;
  animation: app-splash-pulse 1.6s ease-in-out infinite;
}

.app-splash-name {
  font-size: 28px;
}

@keyframes app-splash-pulse {
  0%,
  100% {
    transform: scale(1);
    filter: drop-shadow(0 8px 24px rgba(76, 95, 239, 0.25));
  }

  50% {
    transform: scale(1.05);
    filter: drop-shadow(0 12px 32px rgba(76, 95, 239, 0.45));
  }
}

@media (prefers-reduced-motion: reduce) {
  .app-splash-logo {
    animation: none;
  }
}
</style>
