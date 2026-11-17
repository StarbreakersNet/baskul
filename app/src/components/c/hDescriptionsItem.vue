<script setup lang="ts">
import { NDescriptionsItem, type DescriptionItemProps, type DescriptionItemSlots } from "naive-ui";

// Marque cherchée par NDescriptions sur ses enfants
defineOptions({ DESCRIPTION_ITEM_FLAG: true, inheritAttrs: false });

defineProps</* @vue-ignore */ DescriptionItemProps>();

defineSlots<DescriptionItemSlots>();
</script>

<template>
  <n-descriptions-item v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof DescriptionItemSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-descriptions-item>
</template>
