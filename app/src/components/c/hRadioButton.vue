<script setup lang="ts">
import { NRadioButton, type RadioButtonProps } from "naive-ui";

// NRadioGroup reconnaît ses boutons à ce nom : séparateurs et bordure du bouton choisi
defineOptions({ name: "RadioButton", inheritAttrs: false });

defineProps</* @vue-ignore */ RadioButtonProps>();
</script>

<template>
  <n-radio-button v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </n-radio-button>
</template>
