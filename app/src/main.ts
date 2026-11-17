import { createPinia } from "pinia";
import { createApp } from "vue";
import App from "@/App.vue";
import { router } from "@/router";
import { i18n } from "@/utils/locale";
import "@/assets/main.scss";

async function start() {
  // yarn dev:mock : faux Baskul, pour faire tourner le front seul dans un navigateur
  if (import.meta.env.MODE === "mock") {
    const { installTauriMock } = await import("@/mocks/tauri");
    installTauriMock();
  }
  createApp(App).use(createPinia()).use(i18n).use(router).mount("#app");
}

void start();
