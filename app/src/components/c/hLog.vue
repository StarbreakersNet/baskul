<script setup lang="ts">
import { NLog, type LogInst, type LogProps } from "naive-ui";
import { useTemplateRef } from "vue";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ LogProps>();

const logRef = useTemplateRef<LogInst>("logRef");

function scrollToBottom() {
  logRef.value?.scrollTo({ position: "bottom", silent: true });
}

defineExpose({ scrollToBottom });
</script>

<template>
  <n-log ref="logRef" v-bind="$attrs" />
</template>
