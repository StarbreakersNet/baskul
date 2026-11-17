<script setup lang="ts">
import { NTag, type TagProps, type TagSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ TagProps>();

defineSlots<TagSlots>();
</script>

<template>
  <n-tag v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof TagSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-tag>
</template>
