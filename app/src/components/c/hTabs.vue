<script setup lang="ts">
import { NTabs, type TabsProps, type TabsSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ TabsProps>();

defineSlots<TabsSlots>();
</script>

<template>
  <n-tabs v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof TabsSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-tabs>
</template>
