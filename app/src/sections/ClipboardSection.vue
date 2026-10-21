<script setup lang="ts">
import { NSwitch } from "naive-ui";
import NumberInput from "../components/NumberInput.vue";
import SettingRow from "../components/SettingRow.vue";
import SettingsSection from "../components/SettingsSection.vue";
import type { BasculeConfig } from "../types";

defineProps<{ config: BasculeConfig }>();
</script>

<template>
  <settings-section title="Presse-papiers">
    <setting-row label="Partager le presse-papiers">
      <n-switch v-model:value="config.clipboard.enabled" />
    </setting-row>
    <setting-row label="Images" :disabled="!config.clipboard.enabled">
      <n-switch v-model:value="config.clipboard.images" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row
      label="Texte mis en forme"
      help="Conserve la mise en forme (HTML) quand elle est disponible."
      :disabled="!config.clipboard.enabled"
    >
      <n-switch v-model:value="config.clipboard.html" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row
      label="Ignorer les contenus sensibles"
      help="Mots de passe copiés depuis un gestionnaire de mots de passe."
      :disabled="!config.clipboard.enabled"
    >
      <n-switch v-model:value="config.clipboard.skip_sensitive" :disabled="!config.clipboard.enabled" />
    </setting-row>
    <setting-row label="Taille maximale" :disabled="!config.clipboard.enabled">
      <number-input
        v-model="config.clipboard.max_size_mb"
        :min="1"
        :max="1024"
        :precision="0"
        :disabled="!config.clipboard.enabled"
        suffix="Mo"
      />
    </setting-row>
  </settings-section>
</template>
