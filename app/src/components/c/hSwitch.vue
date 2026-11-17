<script setup lang="ts">
import { NSwitch, type SwitchProps, type SwitchSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ SwitchProps>();

defineSlots<SwitchSlots>();
</script>

<template>
  <n-switch v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof SwitchSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-switch>
</template>
