import { createRouter, createWebHashHistory } from "vue-router";
import AboutView from "@/views/AboutView.vue";
import HomeView from "@/views/HomeView.vue";
import LogView from "@/views/LogView.vue";
import SettingsView from "@/views/SettingsView.vue";

declare module "vue-router" {
  interface RouteMeta {
    /** Clé de traduction du titre affiché dans l'en-tête */
    title: string;
    /** La page a besoin que Baskul réponde */
    needsDaemon: boolean;
  }
}

// Historique "hash" : Tauri sert les pages sans serveur
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
      meta: { title: "nav.home", needsDaemon: true },
    },
    {
      path: "/reglages",
      name: "settings",
      component: SettingsView,
      meta: { title: "nav.settings", needsDaemon: true },
    },
    {
      path: "/journal",
      name: "log",
      component: LogView,
      meta: { title: "nav.log", needsDaemon: true },
    },
    {
      path: "/a-propos",
      name: "about",
      component: AboutView,
      meta: { title: "nav.about", needsDaemon: false },
    },
    { path: "/:rest(.*)*", redirect: { name: "home" } },
  ],
});
