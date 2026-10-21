<script setup lang="ts">
// Carte regroupant des réglages. `machine` : machine concernée par toute la
// section ; les lignes sont grisées si ce n'est pas celle-ci.
import { computed, inject, provide } from "vue";
import { NCard, NTag } from "naive-ui";
import { machineLabel, roleKey, sectionMachineKey } from "../keys";
import type { Role } from "../types";

const props = defineProps<{ title: string; machine?: Role }>();
provide(sectionMachineKey, props.machine);
const role = inject(roleKey);
const other = computed(() => props.machine !== undefined && role?.value !== props.machine);
</script>

<template>
  <n-card class="section" :bordered="true" size="medium">
    <template #header>
      <span class="section-title">
        {{ title }}
        <n-tag
          v-if="machine"
          size="small"
          :type="other ? 'default' : 'info'"
          :bordered="other"
          :title="other ? 'Ces réglages servent sur l\'autre machine.' : undefined"
        >
          {{ machineLabel(machine) }}
        </n-tag>
      </span>
    </template>
    <p v-if="$slots.intro" class="section-intro"><slot name="intro" /></p>
    <slot />
  </n-card>
</template>
