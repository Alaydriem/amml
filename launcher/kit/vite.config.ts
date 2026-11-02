import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { fileURLToPath } from "node:url";

// The gallery is the kit's reference build: no SvelteKit, no Tauri, no network.
// Building it proves every component and screen compiles.
export default defineConfig({
    plugins: [svelte()],
    server: { port: 1440, strictPort: false },
    build: {
        outDir: fileURLToPath(new URL("./dist", import.meta.url)),
        emptyOutDir: true,
        target: ["es2022", "chrome105"],
    },
});
