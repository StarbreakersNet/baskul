<script setup lang="ts">
import { NFlex, type FlexProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ FlexProps>();
</script>

<template>
  <n-flex v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-flex>
</template>
