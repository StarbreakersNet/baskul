<script setup lang="ts">
// Une ligne de réglage : libellé et aide à gauche, contrôle à droite.
// `machine` : la ligne n'apparaît que pour ce rôle.
// `os` : réglage propre à un système ; ailleurs, la ligne est grisée.
import { computed, inject } from "vue";
import { NTag } from "naive-ui";
import { osKey, roleKey } from "../keys";
import type { Role } from "../types";
import { osLabel } from "../utils";

const props = defineProps<{ label: string; help?: string; machine?: Role; os?: string; disabled?: boolean }>();
const role = inject(roleKey);
const currentOs = inject(osKey);
const shown = computed(() => !props.machine || role?.value === props.machine);
const otherOs = computed(() => !!props.os && currentOs?.value !== props.os);
</script>

<template>
  <div v-if="shown" class="row" :class="{ off: disabled || otherOs }">
    <div class="row-text">
      <div class="row-label">
        <span>{{ label }}</span>
        <n-tag v-if="os" size="small" :bordered="false" :type="otherOs ? 'default' : 'info'">
          {{ otherOs ? `${osLabel(os)} uniquement` : osLabel(os) }}
        </n-tag>
      </div>
      <p v-if="help || $slots.help" class="row-help">
        <slot name="help">{{ help }}</slot>
      </p>
    </div>
    <div class="row-control" :inert="otherOs || undefined"><slot /></div>
  </div>
</template>
