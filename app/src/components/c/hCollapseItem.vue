<script setup lang="ts">
import { NCollapseItem, type CollapseItemProps, type CollapseItemSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ CollapseItemProps>();

defineSlots<CollapseItemSlots>();
</script>

<template>
  <n-collapse-item v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof CollapseItemSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-collapse-item>
</template>
