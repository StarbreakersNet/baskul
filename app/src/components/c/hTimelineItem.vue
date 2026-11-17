<script setup lang="ts">
import { NTimelineItem, type TimelineItemProps, type TimelineItemSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ TimelineItemProps>();

defineSlots<TimelineItemSlots>();
</script>

<template>
  <n-timeline-item v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof TimelineItemSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-timeline-item>
</template>
