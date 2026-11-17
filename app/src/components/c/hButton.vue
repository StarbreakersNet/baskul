<script setup lang="ts">
import { NButton, type ButtonProps, type ButtonSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ ButtonProps>();

defineSlots<ButtonSlots>();
</script>

<template>
  <n-button v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof ButtonSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-button>
</template>
