import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
    plugins: [svelte()],
    resolve: {
        // Without the browser condition Svelte resolves to its server build and mount() throws.
        conditions: ["browser"],
    },
    test: {
        environment: "happy-dom",
        globals: true,
        setupFiles: ["./tests/setup.ts"],
        include: ["tests/**/*.test.ts"],
    },
});
