import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Configuration recommandée par Tauri : port fixe, et pas de rechargement
// quand le code Rust change (Tauri s'en charge).
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
    // Les traductions (locales/*.json) sont à la racine du dépôt, partagées
    // avec Bascule.
    fs: { allow: [".."] },
  },
  // vue-i18n : API Composition seulement.
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // WebView2 (Windows), WebKit (macOS 11+) et WebKitGTK (Linux).
    target: ["es2021", "chrome105", "safari15"],
    // App de bureau chargée en local : la taille du bundle importe peu.
    chunkSizeWarningLimit: 1500,
  },
});
