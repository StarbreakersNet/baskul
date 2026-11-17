<script setup lang="ts">
import { NH2, type H2Props } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ H2Props>();
</script>

<template>
  <n-h-2 v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-h-2>
</template>
