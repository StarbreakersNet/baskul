<script setup lang="ts">
import { computed, inject, type Component } from "vue";
import { useThemeVars } from "naive-ui";
import HFlex from "@/components/c/hFlex.vue";
import HGrid from "@/components/c/hGrid.vue";
import HGridItem from "@/components/c/hGridItem.vue";
import HIcon from "@/components/c/hIcon.vue";
import HTag from "@/components/c/hTag.vue";
import HText from "@/components/c/hText.vue";
import ExpandTransition from "@/components/ExpandTransition.vue";
import { OS_KEY, ROLE_KEY } from "@/utils/injectionKeys";
import { osLabel } from "@/utils/os";
import type { Role } from "@/types/Config";

const {
  label,
  icon,
  help,
  machine,
  os,
  disabled = false,
  show = true,
} = defineProps<{
  label: string;
  icon?: Component;
  help?: string;
  /** La ligne n'apparaît que pour ce rôle */
  machine?: Role;
  /** Réglage propre à ce système : grisé sur les autres */
  os?: string;
  disabled?: boolean;
  /** À la place d'un `v-if` sur la ligne : elle apparaît et disparaît en s'animant */
  show?: boolean;
}>();

defineSlots<{
  default(): unknown;
  help?(): unknown;
}>();

const role = inject(ROLE_KEY);
const currentOs = inject(OS_KEY);
const themeVars = useThemeVars();

const isShown = computed(() => show && (!machine || role?.value === machine));
const isOtherOs = computed(() => !!os && currentOs?.value !== os);
const osName = computed(() => osLabel(os ?? ""));
const dividerColor = computed(() => themeVars.value.dividerColor);
const primaryColor = computed(() => themeVars.value.primaryColor);
</script>

<template>
  <expand-transition>
    <div v-if="isShown" :class="['setting-row', { 'setting-row--off': disabled || isOtherOs }]">
      <h-grid
        cols="1 560:2"
        item-style="align-self: center"
        responsive="self"
        :x-gap="24"
        :y-gap="8">
        <h-grid-item>
          <h-flex align="start" :size="12" :wrap="false">
            <span v-if="icon" class="setting-row-icon">
              <h-icon :component="icon" :size="18" />
            </span>
            <div class="setting-row-text">
              <h-flex align="center" :size="8">
                <h-text strong>{{ label }}</h-text>
                <h-tag
                  v-if="os"
                  :bordered="false"
                  size="small"
                  :type="isOtherOs ? 'default' : 'info'">
                  {{ isOtherOs ? $t("common.osOnly", { os: osName }) : osName }}
                </h-tag>
              </h-flex>
              <h-text v-if="help || $slots.help" class="help-text" depth="3" tag="div">
                <slot name="help">{{ help }}</slot>
              </h-text>
            </div>
          </h-flex>
        </h-grid-item>
        <h-grid-item>
          <h-flex align="center" :inert="isOtherOs || undefined" :size="8">
            <slot />
          </h-flex>
        </h-grid-item>
      </h-grid>
    </div>
  </expand-transition>
</template>

<style lang="scss" scoped>
@use "@/assets/variables";

.setting-row {
  padding: 12px 0;
  transition: opacity 0.15s;

  & + & {
    border-top: 1px solid v-bind(dividerColor);
  }
}

.setting-row-icon {
  display: inline-flex;
  flex: none;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: v-bind("themeVars.borderRadius");
  background: color-mix(in srgb, v-bind(primaryColor) variables.$accent-tint, transparent);
  color: v-bind(primaryColor);
}

.setting-row-text {
  min-width: 0;
  padding-top: 6px;
}

.setting-row--off {
  opacity: 0.45;
}
</style>
