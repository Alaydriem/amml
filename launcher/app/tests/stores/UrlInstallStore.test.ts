import { describe, expect, it, vi } from "vitest";
import type { UrlCheckView } from "#lib/bindings/UrlCheckView.ts";
import { UrlInstallStore, type UrlInstallApi } from "#lib/stores/UrlInstallStore.svelte.ts";
import { FakeRegistryEvents } from "../support/FakeRegistryEvents.ts";
import { Samples } from "../support/Samples.ts";

const unlisted: UrlCheckView = {
    id: "@someone/nightvision",
    title: "Night Vision Plus",
    version: null,
    api: 2,
    apiMatches: true,
    matchedRegistry: null,
    hasChecksum: false,
};

function api(check: UrlCheckView, overrides: Partial<UrlInstallApi> = {}): UrlInstallApi {
    return {
        registriesGet: vi.fn(async () => []),
        instanceGet: vi.fn(async () => Samples.instance("play")),
        urlInspect: vi.fn(async () => check),
        urlInstall: vi.fn(async () => 8),
        ...overrides,
    };
}

describe("UrlInstallStore", () => {
    it("load reads the stored registries and follows the event", async () => {
        const events = new FakeRegistryEvents();
        const held = { id: "r1", base: "https://r1/", name: "r1", official: true, status: "ok" as const, modCount: 3, fetched: 1 };
        const registriesGet = vi.fn(async () => [held]);
        const store = new UrlInstallStore("play", api(unlisted, { registriesGet }), vi.fn(), events);

        await store.load();
        expect(registriesGet).toHaveBeenCalledTimes(1);
        expect(store.registryViews).toHaveLength(1);
        events.emit([held, { ...held, id: "r2" }]);
        expect(store.registryViews).toHaveLength(2);
        store.stopWatching();
        expect(events.listeners).toBe(0);
    });

    it("marks a URL that no registry lists as unofficial and installs it only with consent", async () => {
        const urlInstall = vi.fn(async () => 8);
        const navigate = vi.fn();
        const store = new UrlInstallStore("play", api(unlisted, { urlInstall }), navigate, new FakeRegistryEvents());
        store.setUrl("  https://example.org/n.zip ");

        await store.inspect("https://example.org/n.zip");

        expect(store.unofficial).toBe(true);
        expect(store.checks.find((row) => row.id === "registry")?.value).toBe("no checksum");
        expect(store.found).toEqual({ label: "Night Vision Plus", id: "@someone/nightvision" });
        await store.install();
        expect(urlInstall).toHaveBeenCalledExactlyOnceWith("play", "https://example.org/n.zip", true);
        expect(navigate).toHaveBeenCalledWith("/instances/install");
    });

    it("installs a zip a registry lists without the unofficial consent", async () => {
        const urlInstall = vi.fn(async () => 8);
        const listed = { ...unlisted, matchedRegistry: "https://r/", hasChecksum: true, version: "1.2.0" };
        const store = new UrlInstallStore("play", api(listed, { urlInstall }), vi.fn(), new FakeRegistryEvents());

        await store.inspect("https://example.org/n.zip");

        expect(store.unofficial).toBe(false);
        expect(store.found?.label).toBe("Night Vision Plus 1.2.0");
        await store.install();
        expect(urlInstall).toHaveBeenCalledExactlyOnceWith("play", "https://example.org/n.zip", false);
    });

    it("does not install before a check, or a mod for another API, and drops the checks of an old link", async () => {
        const urlInstall = vi.fn(async () => 8);
        const store = new UrlInstallStore("play", api({ ...unlisted, apiMatches: false }, { urlInstall }), vi.fn(), new FakeRegistryEvents());

        await store.install();
        await store.inspect("https://example.org/n.zip");
        expect(store.found).toBeUndefined();
        await store.install();
        expect(urlInstall).not.toHaveBeenCalled();

        store.setUrl("https://example.org/other.zip");
        expect(store.check).toBeNull();
        expect(store.checks).toEqual([]);
    });

    it("shows the fault of a link that cannot be read", async () => {
        const store = new UrlInstallStore(
            "play",
            api(unlisted, {
                urlInspect: vi.fn(async () => {
                    throw { kind: "operation-failed", code: "not a zip", values: {}, restored: false };
                }),
            }),
            vi.fn(),
            new FakeRegistryEvents(),
        );

        await store.inspect("https://example.org/n.zip");

        expect(store.fault?.code).toBe("not a zip");
        expect(store.checking).toBe(false);
        expect(store.check).toBeNull();
    });
});
