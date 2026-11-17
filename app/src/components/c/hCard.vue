<script setup lang="ts">
import { NCard, type CardProps, type CardSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ CardProps>();

defineSlots<CardSlots>();
</script>

<template>
  <n-card v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof CardSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-card>
</template>
