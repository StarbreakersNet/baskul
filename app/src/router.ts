// Pages de l'application. L'historique « hash » convient à Tauri (pages
// servies sans serveur).
import { createRouter, createWebHashHistory } from "vue-router";
import AboutPage from "./pages/AboutPage.vue";
import HomePage from "./pages/HomePage.vue";
import LogPage from "./pages/LogPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";

declare module "vue-router" {
  interface RouteMeta {
    /** Clé de traduction du titre affiché dans l'en-tête. */
    title: string;
    /** La page a besoin que Baskul soit lancé. */
    daemon: boolean;
  }
}

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "home", component: HomePage, meta: { title: "nav.home", daemon: true } },
    { path: "/reglages", name: "settings", component: SettingsPage, meta: { title: "nav.settings", daemon: true } },
    { path: "/journal", name: "log", component: LogPage, meta: { title: "nav.log", daemon: true } },
    { path: "/a-propos", name: "about", component: AboutPage, meta: { title: "nav.about", daemon: false } },
    { path: "/:rest(.*)*", redirect: { name: "home" } },
  ],
});
