<script setup lang="ts">
import { NAlert, type AlertProps, type AlertSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ AlertProps>();

defineSlots<AlertSlots>();
</script>

<template>
  <n-alert v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof AlertSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-alert>
</template>
