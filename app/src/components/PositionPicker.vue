<script setup lang="ts">
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import type { Edge } from "@/types/Config";

const model = defineModel<Edge>({ required: true });

const { disabled = false } = defineProps<{
  disabled?: boolean;
}>();

const EDGES: Edge[] = ["top", "left", "right", "bottom"];

const themeVars = useThemeVars();

const colors = computed(() => ({
  border: themeVars.value.borderColor,
  muted: themeVars.value.textColor3,
  accent: themeVars.value.primaryColor,
  screen: themeVars.value.dividerColor,
}));
</script>

<template>
  <div :aria-label="$t('settings.position.aria')" class="position-picker" role="radiogroup">
    <button
      v-for="edge in EDGES"
      :key="edge"
      :aria-checked="model === edge"
      :class="['position-spot', `position-spot--${edge}`, { 'position-spot--on': model === edge }]"
      :disabled="disabled"
      role="radio"
      type="button"
      @click="model = edge">
      {{ $t(`settings.position.${edge}`) }}
    </button>
    <div class="position-screen">{{ $t("settings.position.thisScreen") }}</div>
  </div>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.position-picker {
  display: grid;
  grid-template-columns: repeat(3, auto);
  gap: 4px;
  justify-content: start;
  align-items: center;
}

.position-spot {
  font: inherit;
  font-size: variables.$font-size-small;
  min-width: 76px;
  padding: 5px 8px;
  border-radius: v-bind("themeVars.borderRadiusSmall");
  border: 1px dashed v-bind("colors.border");
  background: transparent;
  color: v-bind("colors.muted");
  cursor: pointer;

  &:disabled {
    cursor: default;
  }
}

.position-spot--on {
  border: 1px solid v-bind("colors.accent");
  background: color-mix(in srgb, v-bind("colors.accent") variables.$accent-tint, transparent);
  color: v-bind("colors.accent");
  font-weight: 600;
}

.position-spot--top {
  grid-column: 2;
  grid-row: 1;
}

.position-spot--left {
  grid-column: 1;
  grid-row: 2;
}

.position-spot--right {
  grid-column: 3;
  grid-row: 2;
}

.position-spot--bottom {
  grid-column: 2;
  grid-row: 3;
}

.position-screen {
  grid-column: 2;
  grid-row: 2;
  text-align: center;
  font-size: variables.$font-size-small;
  font-weight: 600;
  padding: 12px 8px;
  border-radius: v-bind("themeVars.borderRadiusSmall");
  background: v-bind("colors.screen");
}
</style>
