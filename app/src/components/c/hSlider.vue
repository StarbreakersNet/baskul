<script setup lang="ts">
import { NSlider, type SliderProps, type SliderSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ SliderProps>();

defineSlots<SliderSlots>();
</script>

<template>
  <n-slider v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof SliderSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-slider>
</template>
