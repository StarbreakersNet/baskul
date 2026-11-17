<script setup lang="ts">
import { NCard, type CardProps, type CardSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ CardProps>();

defineSlots<CardSlots>();
</script>

<template>
  <n-card class="glass" v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof CardSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-card>
</template>

<style lang="scss" scoped>
// Une carte posée dans une carte ne refait pas le flou : il est déjà dessous
.glass.n-card--embedded {
  backdrop-filter: none;
  box-shadow: none;
}
</style>
