<script setup lang="ts">
import { NInput, type InputInst, type InputProps, type InputSlots } from "naive-ui";
import { useTemplateRef } from "vue";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ InputProps>();

defineSlots<InputSlots>();

const inputRef = useTemplateRef<InputInst>("inputRef");

function blur() {
  inputRef.value?.blur();
}

function focus() {
  inputRef.value?.focus();
}

defineExpose({ blur, focus });
</script>

<template>
  <n-input ref="inputRef" v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof InputSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-input>
</template>
