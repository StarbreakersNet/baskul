<script setup lang="ts">
import { NRadioGroup, type RadioGroupProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ RadioGroupProps>();
</script>

<template>
  <n-radio-group v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-radio-group>
</template>
