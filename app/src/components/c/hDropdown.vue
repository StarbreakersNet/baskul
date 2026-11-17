<script setup lang="ts">
import { NDropdown, type DropdownProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ DropdownProps>();
</script>

<template>
  <n-dropdown v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-dropdown>
</template>
