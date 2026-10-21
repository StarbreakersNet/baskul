<script setup lang="ts">
// Une ligne de réglage : libellé et aide à gauche, contrôle à droite.
// `machine` : machine concernée (sinon celle de la section) ; la ligne est
// grisée quand elle ne concerne pas le rôle choisi.
import { computed, inject } from "vue";
import { NTag } from "naive-ui";
import { machineLabel, roleKey, sectionMachineKey } from "../keys";
import type { Role } from "../types";

const props = defineProps<{ label: string; help?: string; machine?: Role; disabled?: boolean }>();
const role = inject(roleKey);
const sectionMachine = inject(sectionMachineKey, undefined);
const target = computed(() => props.machine ?? sectionMachine);
const other = computed(() => target.value !== undefined && role?.value !== target.value);
</script>

<template>
  <div class="row" :class="{ off: disabled, other }">
    <div class="row-text">
      <div class="row-label">
        <span>{{ label }}</span>
        <n-tag v-if="machine" size="small" :type="other ? 'default' : 'info'" :bordered="other">
          {{ machineLabel(machine) }}
        </n-tag>
      </div>
      <p v-if="help || $slots.help" class="row-help">
        <slot name="help">{{ help }}</slot>
      </p>
    </div>
    <div class="row-control"><slot /></div>
  </div>
</template>
