<script setup lang="ts">
import { NSelect, type SelectProps, type SelectSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ SelectProps>();

defineSlots<SelectSlots>();
</script>

<template>
  <n-select v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof SelectSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-select>
</template>
