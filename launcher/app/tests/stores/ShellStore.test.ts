import { describe, expect, it, vi } from "vitest";
import { ShellStore, type ShellApi } from "#lib/stores/ShellStore.svelte.ts";

describe("ShellStore", () => {
    it("goes to setup when settings do not exist yet", async () => {
        const first = new ShellStore({ setupGet: async () => ({ needed: true }) } as unknown as ShellApi);
        const later = new ShellStore({ setupGet: async () => ({ needed: false }) } as unknown as ShellApi);

        expect(await first.landing()).toBe("/setup/checks");
        expect(await later.landing()).toBe("/instances");
    });

    it("shows how long ago the official registry answered", () => {
        const store = new ShellStore({} as ShellApi, () => 1_000_000 + 4 * 60_000);
        expect(store.registryFetched()).toBeUndefined();
        store.official = { id: "o", base: "https://r/", name: "Official", official: true, status: "ok", modCount: 1, fetched: 1_000_000 };
        expect(store.registryFetched()).toBe("4 min ago");
    });

    it("keeps a status message for three seconds", () => {
        vi.useFakeTimers();
        const store = new ShellStore({} as ShellApi);
        store.say("Copied");
        expect(store.message).toBe("Copied");
        vi.advanceTimersByTime(ShellStore.MESSAGE_MILLIS);
        expect(store.message).toBeUndefined();
        vi.useRealTimers();
    });
});
