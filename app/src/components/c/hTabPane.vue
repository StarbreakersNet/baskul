<script setup lang="ts">
import { NTabPane, type TabPaneProps, type TabPaneSlots } from "naive-ui";

interface Props extends /* @vue-ignore */ TabPaneProps {
  name: string | number;
}

// Marque cherchée par NTabs sur ses enfants
defineOptions({ __TAB_PANE__: true, inheritAttrs: false });

const { name } = defineProps<Props>();

defineSlots<TabPaneSlots>();
</script>

<template>
  <n-tab-pane :name="name" v-bind="$attrs">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName as keyof TabPaneSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-tab-pane>
</template>
