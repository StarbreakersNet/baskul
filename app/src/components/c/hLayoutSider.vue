<script setup lang="ts">
import { NLayoutSider, type LayoutSiderProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LayoutSiderProps>();
</script>

<template>
  <n-layout-sider v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-layout-sider>
</template>
