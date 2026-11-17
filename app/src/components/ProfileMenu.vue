<script setup lang="ts">
import { computed, h, onMounted, ref, watch } from "vue";
import { useDialog, type DropdownOption } from "naive-ui";
import { useI18n } from "vue-i18n";
import { IconCheck, IconDeviceFloppy, IconStack2, IconTrash } from "@tabler/icons-vue";
import HButton from "@/components/c/hButton.vue";
import HDropdown from "@/components/c/hDropdown.vue";
import HIcon from "@/components/c/hIcon.vue";
import HInput from "@/components/c/hInput.vue";
import { useProfilesStore } from "@/stores/profiles";
import { useSettingsStore } from "@/stores/settings";
import { useStatusStore } from "@/stores/status";
import type { Component } from "vue";

const APPLY_PREFIX = "apply:";

const profiles = useProfilesStore();
const settings = useSettingsStore();
const statusStore = useStatusStore();
const dialog = useDialog();
const { t } = useI18n();

const newName = ref("");

const label = computed(() => profiles.active ?? t("profiles.custom"));

const options = computed<DropdownOption[]>(() => {
  const list: DropdownOption[] = profiles.names.map(name => ({
    key: `${APPLY_PREFIX}${name}`,
    label: name,
    icon: name === profiles.active ? renderIcon(IconCheck) : undefined,
  }));
  if (list.length) {
    list.push({ key: "divider", type: "divider" });
  }
  list.push({ key: "save", label: t("profiles.saveCurrent"), icon: renderIcon(IconDeviceFloppy) });
  if (profiles.active) {
    list.push({
      key: "delete",
      label: t("profiles.delete", { name: profiles.active }),
      icon: renderIcon(IconTrash),
    });
  }
  return list;
});

function renderIcon(icon: Component) {
  return () => h(HIcon, null, { default: () => h(icon) });
}

function askName() {
  newName.value = profiles.active ?? "";
  dialog.create({
    title: t("profiles.saveTitle"),
    content: () =>
      h(HInput, {
        "value": newName.value,
        "placeholder": t("profiles.namePlaceholder"),
        "autofocus": true,
        "onUpdate:value": (value: string) => (newName.value = value),
      }),
    positiveText: t("profiles.save"),
    negativeText: t("common.cancel"),
    onPositiveClick: () => {
      if (newName.value.trim()) {
        void profiles.save(newName.value.trim());
      }
      return Boolean(newName.value.trim());
    },
  });
}

function confirmApply(name: string) {
  if (settings.isDirty) {
    dialog.warning({
      title: t("profiles.applyTitle", { name }),
      content: t("profiles.applyDirty"),
      positiveText: t("profiles.apply"),
      negativeText: t("common.cancel"),
      onPositiveClick: () => void profiles.apply(name),
    });
  } else {
    void profiles.apply(name);
  }
}

function confirmDelete(name: string) {
  dialog.warning({
    title: t("profiles.delete", { name }),
    content: t("profiles.deleteText"),
    positiveText: t("profiles.deleteConfirm"),
    negativeText: t("common.cancel"),
    onPositiveClick: () => void profiles.remove(name),
  });
}

function select(key: string) {
  if (key.startsWith(APPLY_PREFIX)) {
    confirmApply(key.slice(APPLY_PREFIX.length));
  } else if (key === "save") {
    askName();
  } else if (key === "delete" && profiles.active) {
    confirmDelete(profiles.active);
  }
}

watch(() => statusStore.status?.started, profiles.refresh);
onMounted(profiles.refresh);
</script>

<template>
  <h-dropdown :options="options" placement="bottom-end" trigger="click" @select="select($event)">
    <h-button :loading="profiles.applying.loading" round secondary size="small">
      <template #icon>
        <h-icon><icon-stack2 /></h-icon>
      </template>
      <transition mode="out-in" name="fade">
        <span :key="label">{{ label }}</span>
      </transition>
    </h-button>
  </h-dropdown>
</template>
