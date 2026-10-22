<script setup lang="ts">
// Rappel : la version Linux ne partage pas le clavier et la souris.
import { inject } from "vue";
import { NAlert } from "naive-ui";
import { osKey } from "../keys";

const os = inject(osKey);
</script>

<template>
  <n-alert v-if="os === 'linux'" type="info" :bordered="false" class="pane-alert">
    Sous Linux, Bascule ne partage pas encore le clavier et la souris : ces réglages n'ont pas d'effet ici.
  </n-alert>
</template>
