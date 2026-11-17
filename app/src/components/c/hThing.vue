<script setup lang="ts">
import { NThing, type ThingProps, type ThingSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ ThingProps>();

defineSlots<ThingSlots>();
</script>

<template>
  <n-thing v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof ThingSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-thing>
</template>
