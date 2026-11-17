<script setup lang="ts">
import { NInputNumber, type InputNumberProps, type InputNumberSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ InputNumberProps>();

defineSlots<InputNumberSlots>();
</script>

<template>
  <n-input-number v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof InputNumberSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-input-number>
</template>
