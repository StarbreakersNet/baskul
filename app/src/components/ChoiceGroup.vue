<script setup lang="ts" generic="T extends string">
import HRadioButton from "@/components/c/hRadioButton.vue";
import HRadioGroup from "@/components/c/hRadioGroup.vue";
import type { ChoiceOption } from "@/types/Choice";

const model = defineModel<T>({ required: true });

const { options, disabled = false } = defineProps<{
  options: ChoiceOption<T>[];
  disabled?: boolean;
}>();

function select(value: T) {
  model.value = value;
}
</script>

<template>
  <h-radio-group :disabled="disabled" :value="model" @update:value="select($event)">
    <h-radio-button
      v-for="option in options"
      :key="option.value"
      :disabled="option.disabled"
      :value="option.value">
      {{ option.label }}
    </h-radio-button>
  </h-radio-group>
</template>
