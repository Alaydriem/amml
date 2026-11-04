import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
    plugins: [svelte({ compilerOptions: { runes: true } })],
    resolve: {
        // Without the browser condition Svelte resolves to its server build and mount() throws.
        conditions: ["browser"],
        dedupe: ["svelte"],
    },
    test: {
        environment: "happy-dom",
        globals: true,
        setupFiles: ["./tests/setup.ts"],
        include: ["tests/**/*.test.ts"],
    },
});
