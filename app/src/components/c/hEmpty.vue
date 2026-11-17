<script setup lang="ts">
import { NEmpty, type EmptyProps, type EmptySlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ EmptyProps>();

defineSlots<EmptySlots>();
</script>

<template>
  <n-empty v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof EmptySlots" v-bind="slotProps ?? {}" />
    </template>
  </n-empty>
</template>
