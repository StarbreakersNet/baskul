<script setup lang="ts">
import { NProgress, type ProgressProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ ProgressProps>();
</script>

<template>
  <n-progress v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-progress>
</template>
