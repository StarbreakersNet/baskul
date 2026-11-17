import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Port fixe et pas de rechargement sur un changement du Rust : Tauri s'en charge
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
      "@locales": fileURLToPath(new URL("../locales", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
    // Les traductions sont dans locales/, à la racine du dépôt, partagées avec le Rust
    fs: { allow: [".."] },
  },
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false,
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    // WebView2 (Windows), WebKit (macOS 11+) et WebKitGTK (Linux)
    target: ["es2021", "chrome105", "safari15"],
    chunkSizeWarningLimit: 1500,
  },
});
