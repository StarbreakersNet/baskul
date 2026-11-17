<script setup lang="ts">
import { NCollapse, type CollapseProps, type CollapseSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ CollapseProps>();

defineSlots<CollapseSlots>();
</script>

<template>
  <n-collapse v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof CollapseSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-collapse>
</template>
