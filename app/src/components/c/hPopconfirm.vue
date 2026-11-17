<script setup lang="ts">
import { NPopconfirm, type PopconfirmProps, type PopconfirmSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ PopconfirmProps>();

defineSlots<PopconfirmSlots>();
</script>

<template>
  <n-popconfirm v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof PopconfirmSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-popconfirm>
</template>
