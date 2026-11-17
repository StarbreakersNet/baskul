<script setup lang="ts">
import { NTooltip, type TooltipProps, type TooltipSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ TooltipProps>();

defineSlots<TooltipSlots>();
</script>

<template>
  <n-tooltip v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof TooltipSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-tooltip>
</template>
