<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";

const themeVars = useThemeVars();

// Du survol à l'appui de l'accent : le dégradé du logo, avec les couleurs du thème
const gradient = computed(
  () =>
    `linear-gradient(135deg, ${themeVars.value.primaryColorHover} 0%, ${themeVars.value.primaryColorPressed} 100%)`
);
</script>

<template>
  <span class="brand-name">Baskul</span>
</template>

<style lang="scss" scoped>
.brand-name {
  background: v-bind(gradient);
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  font-weight: 700;
  letter-spacing: -0.01em;
}
</style>
