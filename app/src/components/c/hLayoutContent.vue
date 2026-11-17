<script setup lang="ts">
import { NLayoutContent, type LayoutContentInst, type LayoutContentProps } from "naive-ui";
import { useTemplateRef } from "vue";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LayoutContentProps>();

const contentRef = useTemplateRef<LayoutContentInst>("contentRef");

function scrollToTop() {
  contentRef.value?.scrollTo({ top: 0 });
}

defineExpose({ scrollToTop });
</script>

<template>
  <n-layout-content ref="contentRef" v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-layout-content>
</template>
