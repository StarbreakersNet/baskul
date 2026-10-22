<script setup lang="ts">
// Champ numérique : un champ vidé garde la valeur précédente.
import { NInputNumber } from "naive-ui";

const model = defineModel<number>({ required: true });
defineProps<{ min?: number; max?: number; step?: number; precision?: number; disabled?: boolean; suffix?: string }>();
</script>

<template>
  <n-input-number
    :value="model"
    :min="min"
    :max="max"
    :step="step"
    :precision="precision"
    :disabled="disabled"
    style="width: 150px"
    @update:value="(v) => v !== null && (model = v)"
  >
    <template v-if="suffix" #suffix>{{ suffix }}</template>
  </n-input-number>
</template>
