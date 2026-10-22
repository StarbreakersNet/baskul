<script setup lang="ts">
// Position de l'écran de l'autre machine autour de celui-ci.
import { computed } from "vue";
import { useThemeVars } from "naive-ui";
import type { Edge } from "../types";

const model = defineModel<Edge>({ required: true });
defineProps<{ disabled?: boolean }>();

const vars = useThemeVars();
const colors = computed(() => ({
  border: vars.value.borderColor,
  muted: vars.value.textColor3,
  accent: vars.value.primaryColor,
  soft: `color-mix(in srgb, ${vars.value.primaryColor} 14%, transparent)`,
  screen: vars.value.dividerColor,
}));

const spots: { edge: Edge; label: string }[] = [
  { edge: "top", label: "En haut" },
  { edge: "left", label: "À gauche" },
  { edge: "right", label: "À droite" },
  { edge: "bottom", label: "En bas" },
];
</script>

<template>
  <div class="pos" role="radiogroup" aria-label="Position de l'autre écran">
    <button
      v-for="s in spots"
      :key="s.edge"
      type="button"
      role="radio"
      :aria-checked="model === s.edge"
      :class="['spot', `p-${s.edge}`, { on: model === s.edge }]"
      :disabled="disabled"
      @click="model = s.edge"
    >
      {{ s.label }}
    </button>
    <div class="me">Cet écran</div>
  </div>
</template>

<style scoped>
.pos {
  display: grid;
  grid-template-columns: repeat(3, auto);
  gap: 4px;
  justify-content: start;
  align-items: center;
}
.spot {
  font: inherit;
  font-size: 12.5px;
  min-width: 76px;
  padding: 5px 8px;
  border-radius: 7px;
  border: 1px dashed v-bind("colors.border");
  background: transparent;
  color: v-bind("colors.muted");
  cursor: pointer;
}
.spot.on {
  border: 1px solid v-bind("colors.accent");
  background: v-bind("colors.soft");
  color: v-bind("colors.accent");
  font-weight: 600;
}
.spot:disabled {
  cursor: default;
}
.me {
  grid-column: 2;
  grid-row: 2;
  text-align: center;
  font-size: 12.5px;
  font-weight: 600;
  padding: 12px 8px;
  border-radius: 7px;
  background: v-bind("colors.screen");
}
.p-top {
  grid-column: 2;
  grid-row: 1;
}
.p-left {
  grid-column: 1;
  grid-row: 2;
}
.p-right {
  grid-column: 3;
  grid-row: 2;
}
.p-bottom {
  grid-column: 2;
  grid-row: 3;
}
</style>
