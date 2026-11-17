<script setup lang="ts">
import { NGrid, type GridProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ GridProps>();
</script>

<template>
  <n-grid v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-grid>
</template>
