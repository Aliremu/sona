import { defineConfig } from "vite";
import path from "path"
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite"
import { readFileSync } from "fs";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Read package.json to get app metadata
const packageJson = JSON.parse(readFileSync("./package.json", "utf-8"));

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  define: {
    // Inject app constants at build time
    __APP_NAME__: JSON.stringify(packageJson.name),
    __APP_VERSION__: JSON.stringify(packageJson.version),
    __APP_DESCRIPTION__: JSON.stringify(packageJson.description || "Professional music practice software with VST support"),
    __APP_AUTHOR__: JSON.stringify(packageJson.author || "William Zhang"),
  },

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
