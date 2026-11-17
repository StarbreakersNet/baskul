import { computed, reactive, ref } from "vue";

export type LoaderInstance = ReturnType<typeof useLoader>;

export function useLoader() {
  const count = ref(0);

  const loading = computed(() => count.value > 0);

  function start() {
    count.value++;
  }

  function stop() {
    if (count.value > 0) {
      count.value--;
    }
  }

  function reset() {
    count.value = 0;
  }

  async function run<T>(task: () => Promise<T>): Promise<T> {
    start();
    try {
      return await task();
    } finally {
      stop();
    }
  }

  // reactive : un computed rangé dans un objet ordinaire ne serait pas déballé par le template
  return reactive({
    count,
    loading,
    start,
    stop,
    reset,
    run,
  });
}

export function isAnyLoading(...loaders: LoaderInstance[]) {
  return loaders.some(loader => loader.loading);
}
