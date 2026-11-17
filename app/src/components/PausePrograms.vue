<script setup lang="ts">
import { computed } from "vue";
import { IconDeviceGamepad2 } from "@tabler/icons-vue";
import HButton from "@/components/c/hButton.vue";
import HDynamicTags from "@/components/c/hDynamicTags.vue";
import HFlex from "@/components/c/hFlex.vue";
import SettingRow from "@/components/SettingRow.vue";
import { useStatusStore } from "@/stores/status";

const programs = defineModel<string[]>({ required: true });

const statusStore = useStatusStore();

const foreground = computed(() => {
  const name = statusStore.status?.foreground ?? null;
  const isListed = programs.value.some(entry => sameProgram(entry, name ?? ""));
  return name && !isListed ? name.replace(/\.exe$/i, "") : null;
});

function programKey(name: string) {
  return name
    .trim()
    .toLowerCase()
    .replace(/\.(exe|app)$/, "");
}

function sameProgram(entry: string, name: string) {
  return programKey(entry) === programKey(name);
}

function addForeground() {
  if (foreground.value) {
    programs.value = [...programs.value, foreground.value];
  }
}
</script>

<template>
  <setting-row
    :icon="IconDeviceGamepad2"
    :help="$t('settings.pause.help')"
    :label="$t('settings.pause.label')">
    <h-flex class="pause-programs" :size="8" vertical>
      <h-dynamic-tags v-model:value="programs" />
      <transition name="fade">
        <div v-if="foreground">
          <h-button
            quaternary
            size="tiny"
            :title="$t('settings.pause.addForegroundHint')"
            @click="addForeground()">
            {{ $t("settings.pause.addForeground", { name: foreground }) }}
          </h-button>
        </div>
      </transition>
    </h-flex>
  </setting-row>
</template>

<style lang="scss" scoped>
.pause-programs {
  width: 100%;
}
</style>
