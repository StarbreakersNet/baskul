<script setup lang="ts">
import { NSpin, type SpinProps, type SpinSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ SpinProps>();

defineSlots<SpinSlots>();
</script>

<template>
  <n-spin v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof SpinSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-spin>
</template>
