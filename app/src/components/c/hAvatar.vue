<script setup lang="ts">
import { NAvatar, type AvatarProps, type AvatarSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ AvatarProps>();

defineSlots<AvatarSlots>();
</script>

<template>
  <n-avatar v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof AvatarSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-avatar>
</template>
