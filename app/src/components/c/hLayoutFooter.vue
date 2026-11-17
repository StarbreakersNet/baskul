<script setup lang="ts">
import { NLayoutFooter, type LayoutFooterProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LayoutFooterProps>();
</script>

<template>
  <n-layout-footer v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-layout-footer>
</template>
