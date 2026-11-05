import { describe, expect, it, vi } from "vitest";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { ShellStore, type ShellApi } from "#lib/stores/ShellStore.svelte.ts";
import { FakeRegistryEvents } from "../support/FakeRegistryEvents.ts";

function view(id: string, official = false): RegistryView {
    return { id, base: `https://${id}/`, name: id, official, status: "ok", modCount: 1, fetched: 1 };
}

describe("ShellStore", () => {
    it("goes to setup when settings do not exist yet", async () => {
        const first = new ShellStore({ setupGet: async () => ({ needed: true }) } as unknown as ShellApi, new FakeRegistryEvents());
        const later = new ShellStore({ setupGet: async () => ({ needed: false }) } as unknown as ShellApi, new FakeRegistryEvents());

        expect(await first.landing()).toBe("/setup/checks");
        expect(await later.landing()).toBe("/instances");
    });

    it("shows how long ago the official registry answered", () => {
        const store = new ShellStore({} as ShellApi, new FakeRegistryEvents(), () => 1_000_000 + 4 * 60_000);
        expect(store.registryFetched()).toBeUndefined();
        store.official = { id: "o", base: "https://r/", name: "Official", official: true, status: "ok", modCount: 1, fetched: 1_000_000 };
        expect(store.registryFetched()).toBe("4 min ago");
    });

    it("start reads the stored registries and does not refresh", async () => {
        const registriesGet = vi.fn(async () => [view("friends")]);
        const registriesRefresh = vi.fn(async () => []);
        const events = new FakeRegistryEvents();
        const store = new ShellStore({ registriesGet, registriesRefresh } as unknown as ShellApi, events);

        await store.start();

        expect(registriesRefresh).not.toHaveBeenCalled();
        expect(registriesGet).toHaveBeenCalledTimes(1);
        expect(store.official).toBeNull();
        events.emit([view("friends"), view("official", true)]);
        expect(store.official?.id).toBe("official");
        expect(store.registries).toHaveLength(2);
    });

    it("ignores a stored read that ends after an event and stops listening on dispose", async () => {
        let release: (views: RegistryView[]) => void = () => {};
        const registriesGet = vi.fn(() => new Promise<RegistryView[]>((resolve) => (release = resolve)));
        const events = new FakeRegistryEvents();
        const store = new ShellStore({ registriesGet } as unknown as ShellApi, events);

        const started = store.start();
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
        events.emit([view("official", true)]);
        release([]);
        await started;

        expect(store.official?.id).toBe("official");
        store.dispose();
        expect(events.listeners).toBe(0);
    });

    it("keeps a status message for three seconds", () => {
        vi.useFakeTimers();
        const store = new ShellStore({} as ShellApi, new FakeRegistryEvents());
        store.say("Copied");
        expect(store.message).toBe("Copied");
        vi.advanceTimersByTime(ShellStore.MESSAGE_MILLIS);
        expect(store.message).toBeUndefined();
        vi.useRealTimers();
    });
});
