import { onActivated, onBeforeUnmount, onDeactivated, onMounted, ref } from "vue";

/** Heure actuelle, rafraîchie seulement pendant que le composant est affiché */
export function useCurrentTime(intervalMs: number) {
  const now = ref(Date.now());
  let timer: ReturnType<typeof setInterval> | undefined;

  function start() {
    clearInterval(timer);
    now.value = Date.now();
    timer = setInterval(() => {
      now.value = Date.now();
    }, intervalMs);
  }

  function stop() {
    clearInterval(timer);
  }

  onMounted(start);
  onActivated(start);
  onDeactivated(stop);
  onBeforeUnmount(stop);

  return now;
}
