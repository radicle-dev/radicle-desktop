import path from "node:path";

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

// https://vitejs.dev/config/
export default defineConfig({
  test: {
    environment: "happy-dom",
    include: ["tests/unit/**/*.test.ts"],
    reporters: "verbose",
  },
  plugins: [
    // Asciidoctor derives Node-only LIB_DIR/ROOT_DIR/DATA_DIR from
    // `import.meta.url`. Those paths are unused in the browser, but Vite still
    // emits the whole module as an asset, duplicating it unminified in the
    // bundle. Neutralize the URL so nothing is emitted.
    {
      name: "asciidoctor-drop-node-paths",
      enforce: "pre" as const,
      transform(code: string, id: string) {
        if (!id.includes("@asciidoctor/core")) return;

        const pattern = /new URL\((.*?), import\.meta\.url\)/g;
        const patched = code.replaceAll(pattern, "new URL($1, `file:///`)");
        const expected = code.includes("DATA_DIR") ? 3 : 0;
        const found = code.match(pattern)?.length ?? 0;

        if (found !== expected) {
          throw new Error(
            `Expected ${expected} import.meta.url uses in ${id}, found ${found}. ` +
              `Asciidoctor may now rely on them at runtime; re-check before patching.`,
          );
        }

        return found > 0 ? { code: patched } : undefined;
      },
    },
    svelte({
      // Reference: https://github.com/sveltejs/vite-plugin-svelte/issues/270#issuecomment-1033190138
      dynamicCompileOptions({ filename }) {
        if (path.basename(filename) === "Clipboard.svelte") {
          return { customElement: true };
        }
      },
    }),
  ],
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
