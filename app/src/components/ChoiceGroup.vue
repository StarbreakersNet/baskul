<script setup lang="ts" generic="T extends string">
// Choix exclusif présenté en boutons.
import { NRadioButton, NRadioGroup } from "naive-ui";

const model = defineModel<T>({ required: true });
defineProps<{ options: { value: T; label: string }[]; disabled?: boolean }>();
</script>

<template>
  <n-radio-group :value="model" :disabled="disabled" @update:value="(v) => (model = v as T)">
    <n-radio-button v-for="o in options" :key="o.value" :value="o.value">{{ o.label }}</n-radio-button>
  </n-radio-group>
</template>
