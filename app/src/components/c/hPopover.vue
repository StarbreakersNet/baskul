<script setup lang="ts">
import { NPopover, type PopoverProps, type PopoverSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ PopoverProps>();

defineSlots<PopoverSlots>();
</script>

<template>
  <n-popover v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof PopoverSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-popover>
</template>
