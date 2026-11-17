<script setup lang="ts">
import { NIcon, type IconProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ IconProps>();
</script>

<template>
  <n-icon v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-icon>
</template>
