<script setup lang="ts">
import HInputNumber from "@/components/c/hInputNumber.vue";

const model = defineModel<number>({ required: true });

const {
  min,
  max,
  step,
  precision,
  disabled = false,
  suffix,
} = defineProps<{
  min?: number;
  max?: number;
  step?: number;
  precision?: number;
  disabled?: boolean;
  suffix?: string;
}>();

// Un champ vidé garde la valeur précédente : la configuration n'accepte pas de valeur absente
function update(value: number | null) {
  if (value !== null) {
    model.value = value;
  }
}
</script>

<template>
  <h-input-number
    class="number-input"
    :disabled="disabled"
    :max="max"
    :min="min"
    :precision="precision"
    :step="step"
    :value="model"
    @update:value="update($event)">
    <template v-if="suffix" #suffix>{{ suffix }}</template>
  </h-input-number>
</template>

<style lang="scss" scoped>
.number-input {
  width: 150px;
}
</style>
