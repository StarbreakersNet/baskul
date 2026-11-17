<script setup lang="ts">
import { NSkeleton, type SkeletonProps } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ SkeletonProps>();
</script>

<template>
  <n-skeleton v-bind="$attrs" />
</template>
