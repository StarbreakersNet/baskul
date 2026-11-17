<script setup lang="ts">
import { NDescriptions, type DescriptionsProps, type DescriptionsSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ DescriptionsProps>();

defineSlots<DescriptionsSlots>();
</script>

<template>
  <n-descriptions v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof DescriptionsSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-descriptions>
</template>
