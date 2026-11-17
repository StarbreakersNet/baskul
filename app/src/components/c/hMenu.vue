<script setup lang="ts">
import { NMenu, type MenuProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ MenuProps>();
</script>

<template>
  <n-menu v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-menu>
</template>
