<script setup lang="ts">
import { NLayout, type LayoutProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LayoutProps>();
</script>

<template>
  <n-layout v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-layout>
</template>
