<script setup lang="ts">
import { NTimeline, type TimelineProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ TimelineProps>();
</script>

<template>
  <n-timeline v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-timeline>
</template>
