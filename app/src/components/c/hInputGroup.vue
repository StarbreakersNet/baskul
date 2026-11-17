<script setup lang="ts">
import { NInputGroup, type InputGroupProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ InputGroupProps>();
</script>

<template>
  <n-input-group v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-input-group>
</template>
