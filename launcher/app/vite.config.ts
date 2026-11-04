import adapter from "@sveltejs/adapter-static";
import { sveltekit } from "@sveltejs/kit/vite";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

export default defineConfig({
    plugins: [
        sveltekit({
            preprocess: vitePreprocess(),
            compilerOptions: { runes: true },
            // Tauri has no Node server, so the app is a single-page build that falls back to index.html
            adapter: adapter({ pages: "build", assets: "build", fallback: "index.html" }),
        }),
    ],
    resolve: { dedupe: ["svelte"] },
    clearScreen: false,
    server: { port: 1430, strictPort: true, fs: { allow: [".."] } },
    build: { target: ["es2022", "chrome105"] },
});
