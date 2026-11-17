<script setup lang="ts">
import { NLayoutHeader, type LayoutHeaderProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LayoutHeaderProps>();
</script>

<template>
  <n-layout-header v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-layout-header>
</template>
