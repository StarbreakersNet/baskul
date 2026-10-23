<script setup lang="ts">
// Une ligne de réglage : libellé et aide à gauche, contrôle à droite (une
// seule colonne quand la place manque).
// `machine` : la ligne n'apparaît que pour ce rôle.
// `os` : réglage propre à un système ; ailleurs, la ligne est grisée.
import { computed, inject } from "vue";
import { NFlex, NGi, NGrid, NTag, NText, useThemeVars } from "naive-ui";
import { osKey, roleKey } from "../keys";
import type { Role } from "../types";
import { osLabel } from "../utils";

const props = defineProps<{ label: string; help?: string; machine?: Role; os?: string; disabled?: boolean }>();
const role = inject(roleKey);
const currentOs = inject(osKey);
const shown = computed(() => !props.machine || role?.value === props.machine);
const otherOs = computed(() => !!props.os && currentOs?.value !== props.os);
const vars = useThemeVars();
const divider = computed(() => vars.value.dividerColor);
</script>

<template>
  <div v-if="shown" class="setting" :class="{ off: disabled || otherOs }">
    <n-grid cols="1 560:2" responsive="self" :x-gap="24" :y-gap="8" item-style="align-self: center">
      <n-gi>
        <n-flex align="center" :size="8">
          <n-text strong>{{ label }}</n-text>
          <n-tag v-if="os" size="small" :bordered="false" :type="otherOs ? 'default' : 'info'">
            {{ otherOs ? $t("common.osOnly", { os: osLabel(os) }) : osLabel(os) }}
          </n-tag>
        </n-flex>
        <n-text v-if="help || $slots.help" depth="3" tag="div" class="help">
          <slot name="help">{{ help }}</slot>
        </n-text>
      </n-gi>
      <n-gi>
        <n-flex align="center" :size="8" :inert="otherOs || undefined">
          <slot />
        </n-flex>
      </n-gi>
    </n-grid>
  </div>
</template>

<style scoped>
.setting {
  padding: 12px 0;
  transition: opacity 0.15s;
}
.setting + .setting {
  border-top: 1px solid v-bind(divider);
}
.setting.off {
  opacity: 0.45;
}
.help {
  margin-top: 2px;
  font-size: 13px;
}
</style>
