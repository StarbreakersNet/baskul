<script setup lang="ts">
import { NResult, type ResultProps, type ResultSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ ResultProps>();

defineSlots<ResultSlots>();
</script>

<template>
  <n-result v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof ResultSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-result>
</template>
