<script setup lang="ts">
import { Transition, TransitionGroup } from "vue";

// Hauteur animée de 0 à celle du contenu : ce qui apparaît pousse le reste au lieu de le déplacer d'un coup
const { group = false } = defineProps<{
  /** Liste (`v-for`) plutôt qu'un seul élément */
  group?: boolean;
}>();

function expand(element: Element) {
  const el = element as HTMLElement;
  el.style.height = "0";
  void el.offsetHeight;
  el.style.height = `${el.scrollHeight}px`;
}

function release(element: Element) {
  (element as HTMLElement).style.height = "";
}

function collapse(element: Element) {
  const el = element as HTMLElement;
  el.style.height = `${el.scrollHeight}px`;
  void el.offsetHeight;
  el.style.height = "0";
}
</script>

<template>
  <component
    :is="group ? TransitionGroup : Transition"
    name="expand"
    @after-enter="release($event)"
    @enter="expand($event)"
    @leave="collapse($event)">
    <slot />
  </component>
</template>
