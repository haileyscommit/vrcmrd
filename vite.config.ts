/// <reference types="node" />
import { defineConfig, UserConfigFn, UserConfigFnPromise } from "vite";
import preact from "@preact/preset-vite";
import tailwindcss from '@tailwindcss/vite';
import svgr from 'vite-plugin-svgr';
import { fileURLToPath } from "url";
import path from "path";
import fs from "fs";

const root = path.dirname(fileURLToPath(import.meta.url));

const host = process.env.TAURI_DEV_HOST;

const inputs = Object.fromEntries(
  fs.readdirSync("src/entrypoints")
    .filter(f => f.endsWith(".html"))
    .map(f => [
      f.replace(/\.html$/, "").replace(/src\/entrypoints\//, ""),
      path.join("src/entrypoints", f),
    ])
);

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    preact(),
    tailwindcss(),
    svgr({
      svgrOptions: {
        jsxRuntime: 'classic-preact',
      },
    }),
  ],

  build: {
    rollupOptions: {
      input: inputs,
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
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
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}) as unknown);
