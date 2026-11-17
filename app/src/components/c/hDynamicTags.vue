<script setup lang="ts">
import { NDynamicTags, type DynamicTagsProps, type DynamicTagsSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ DynamicTagsProps>();

defineSlots<DynamicTagsSlots>();
</script>

<template>
  <n-dynamic-tags v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof DynamicTagsSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-dynamic-tags>
</template>
