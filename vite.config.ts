import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "node:path";

// https://vitejs.dev/config/
export default defineConfig({
  test: {
    environment: "happy-dom",
    include: ["tests/unit/**/*.test.ts"],
    reporters: "verbose",
  },
  plugins: [svelte()],
  build: {
    outDir: "build",
  },
  // prevent vite from obscuring rust errors
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/crates/radicle-tauri/**"],
    },
  },
  resolve: {
    alias: {
      "@app": path.resolve("./src"),
      "@bindings": path.resolve("./crates/radicle-types/bindings/"),
    },
  },
});
