<script setup lang="ts">
import { NBadge, type BadgeProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ BadgeProps>();
</script>

<template>
  <n-badge v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-badge>
</template>
