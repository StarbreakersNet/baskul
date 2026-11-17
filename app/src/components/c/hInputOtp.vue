<script setup lang="ts">
import { NInputOtp, type InputOtpProps, type InputOtpSlots } from "naive-ui";

defineOptions({ inheritAttrs: false });

defineProps</* @vue-ignore */ InputOtpProps>();

defineSlots<InputOtpSlots>();
</script>

<template>
  <n-input-otp v-bind="$attrs">
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name as keyof InputOtpSlots" v-bind="slotProps ?? {}" />
    </template>
  </n-input-otp>
</template>
