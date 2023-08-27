//@ts-ignore
import { resolve } from 'path'
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

declare const __dirname: string

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // 3. to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    rollupOptions: {
      input: {
        index: resolve(__dirname, 'src/app/index.html'),
        splashscreen: resolve(__dirname, 'index.html'),
      }
    },
  },
}));
