import { describe, expect, it, vi } from "vitest";
import type { CardView } from "#lib/bindings/CardView.ts";
import type { CatalogLayoutView } from "#lib/bindings/CatalogLayoutView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { CatalogStore, type CatalogApi } from "#lib/stores/CatalogStore.svelte.ts";
import { Samples } from "../support/Samples.ts";

function card(id: string, registryId = "r1", overrides: Partial<CardView> = {}): CardView {
    return {
        id,
        registryId,
        title: id,
        summary: "s",
        tags: [],
        version: "1.0.0",
        size: 1,
        iconUrl: null,
        state: "available",
        updated: "2026-10-01",
        ...overrides,
    };
}

const registry = (id: string, overrides: Partial<RegistryView> = {}): RegistryView => ({
    id,
    base: `https://${id}/`,
    name: id.toUpperCase(),
    official: id === "r1",
    status: "ok",
    modCount: 100,
    fetched: 0,
    ...overrides,
});

function layout(registryId: string, total: number, pageSize = 10): CatalogLayoutView {
    return { registryId, api: 2, pageSize, total, groups: [{ id: "A", label: "A", start: 0, total }] };
}

function api(overrides: Partial<CatalogApi> = {}): CatalogApi {
    return {
        registriesRefresh: vi.fn(async () => [registry("r1"), registry("r2")]),
        instanceGet: vi.fn(async () => Samples.instance("play")),
        catalogLayout: vi.fn(async (_name: string, registryId: string) => layout(registryId, 60)),
        catalogPage: vi.fn(async (_name: string, registryId: string, page: number) => [card(`p${page}-a`, registryId), card(`p${page}-b`, registryId)]),
        catalogSearch: vi.fn(async () => []),
        catalogSorted: vi.fn(async () => null),
        catalogDetail: vi.fn(async () => {
            throw new Error("no detail");
        }),
        modsInstall: vi.fn(async () => 6),
        ...overrides,
    };
}

async function loaded(overrides: Partial<CatalogApi> = {}, navigate = vi.fn()) {
    const store = new CatalogStore("play", api(overrides), navigate);
    await store.load();
    return store;
}

describe("CatalogStore", () => {
    it("opens the first registry and loads its first page", async () => {
        const store = await loaded();

        expect(store.source).toBe("r1");
        expect(store.layout?.total).toBe(60);
        expect(store.allCount).toBe(200);
        expect(store.groups()[0]?.rows.map((row) => row.id)).toEqual(["p1-a", "p1-b"]);
        expect(store.context).toEqual({ game: "1.26.60.05", amml: "26.60.1", api: "2" });
    });

    it("a_failed_page_leaves_loading_and_is_asked_for_again", async () => {
        const catalogPage = vi
            .fn()
            .mockResolvedValueOnce([card("first")])
            .mockRejectedValueOnce(new Error("offline"))
            .mockResolvedValueOnce([card("later")]);
        const store = await loaded({ catalogPage });

        await store.loadPage({ registryId: "r1", page: 2 });
        expect(store.loading).toEqual([]);
        expect(store.pages.r1?.[2]).toBeUndefined();

        await store.loadPage({ registryId: "r1", page: 2 });
        expect(catalogPage).toHaveBeenCalledTimes(3);
        expect(store.pages.r1?.[2]?.map((held) => held.id)).toEqual(["later"]);
    });

    it("asks for a page that is on its way only once", async () => {
        let release: (cards: CardView[]) => void = () => {};
        const slow = new Promise<CardView[]>((resolve) => (release = resolve));
        const catalogPage = vi.fn().mockResolvedValueOnce([card("first")]).mockReturnValueOnce(slow);
        const store = await loaded({ catalogPage });

        const first = store.loadPage({ registryId: "r1", page: 3 });
        const second = store.loadPage({ registryId: "r1", page: 3 });
        expect(store.loading).toHaveLength(1);
        release([card("third")]);
        await Promise.all([first, second]);

        expect(catalogPage).toHaveBeenCalledTimes(2);
    });

    it("keeps at most ten pages per registry and drops the page furthest from the last one asked", async () => {
        const store = await loaded({ catalogLayout: vi.fn(async (_n: string, id: string) => layout(id, 400)) });
        for (let page = 2; page <= 11; page += 1) await store.loadPage({ registryId: "r1", page });
        expect(Object.keys(store.pages.r1 ?? {}).map(Number).sort((a, b) => a - b)).toEqual([2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

        await store.loadPage({ registryId: "r1", page: 12 });

        expect(Object.keys(store.pages.r1 ?? {})).toHaveLength(CatalogStore.MAX_PAGES);
        expect(store.pages.r1?.[2]).toBeUndefined();
        expect(store.pages.r1?.[12]).toBeDefined();
    });

    it("starts a new generation on a new query, source or sort", async () => {
        const store = await loaded();
        const start = store.generation;

        await store.setQuery("cam");
        const afterQuery = store.generation;
        await store.chooseSource("r2");
        const afterSource = store.generation;
        await store.setSort("size");
        const afterSort = store.generation;

        expect(afterQuery).toBeGreaterThan(start);
        expect(afterSource).toBeGreaterThan(afterQuery);
        expect(afterSort).toBeGreaterThan(afterSource);
    });

    it("shows five cards per search group and all of them after show more", async () => {
        const cards = Array.from({ length: 8 }, (_, i) => card(`m${i}`, "r1"));
        const store = await loaded({ catalogSearch: vi.fn(async () => [{ registryId: "r1", total: 8, cards }]) });
        await store.chooseSource("all");
        await store.setQuery("m");

        const [group] = store.groups();
        expect(group?.rows).toHaveLength(5);
        expect(group?.collapsedMore).toBe(3);
        store.showMore("r1");
        const [all] = store.groups();
        expect(all?.rows).toHaveLength(8);
        expect(all?.collapsedMore).toBeUndefined();
        expect(store.resultLine()).toBe("8 results in 1 registries");
    });

    it("counts the matches of the other registries when one registry is chosen", async () => {
        const groups = [
            { registryId: "r1", total: 0, cards: [] },
            { registryId: "r2", total: 4, cards: [card("x", "r2")] },
        ];
        const store = await loaded({ catalogSearch: vi.fn(async () => groups) });

        await store.setQuery("x");

        expect(store.otherMatches()).toBe(4);
        expect(store.groups().map((group) => group.registryId)).toEqual(["r1"]);
    });

    it("sorts by size through the bridge and keeps the name order when it answers none", async () => {
        const catalogSorted = vi.fn().mockResolvedValueOnce([card("big", "r1", { size: 9 }), card("small", "r1", { size: 1 })]).mockResolvedValueOnce(null);
        const store = await loaded({ catalogSorted });

        await store.setSort("size");
        expect(store.groups()[0]?.rows.map((row) => row.id)).toEqual(["big", "small"]);
        expect(catalogSorted).toHaveBeenCalledWith("play", "r1", "size");

        await store.setSort("updated");
        expect(store.groups()[0]?.id).toBe("A");
    });

    it("keeps ticked cards across a new search", async () => {
        const search = vi
            .fn()
            .mockResolvedValueOnce([{ registryId: "r1", total: 1, cards: [card("one", "r1")] }])
            .mockResolvedValueOnce([{ registryId: "r1", total: 1, cards: [card("two", "r1")] }]);
        const store = await loaded({ catalogSearch: search });
        await store.chooseSource("all");
        await store.setQuery("one");
        store.setChecked(["one"]);

        await store.setQuery("two");
        store.setChecked(["one", "two"]);

        expect(store.selected.map((held) => held.id)).toEqual(["one", "two"]);
        store.setChecked(["two"]);
        expect(store.checked).toEqual(["two"]);
        store.clearSelection();
        expect(store.selected).toEqual([]);
    });

    it("installs the ticked mods and opens the install screen", async () => {
        const modsInstall = vi.fn(async () => 6);
        const navigate = vi.fn();
        const store = await loaded({ modsInstall }, navigate);
        store.setChecked(["p1-a", "p1-b"]);

        await store.install();

        expect(modsInstall).toHaveBeenCalledExactlyOnceWith("play", ["p1-a", "p1-b"]);
        expect(navigate).toHaveBeenCalledWith("/instances/install");
    });

    it("shows the detail pane at once and fills it when the document comes", async () => {
        let release: (view: never) => void = () => {};
        const catalogDetail = vi.fn(() => new Promise<never>((resolve) => (release = resolve)));
        const store = await loaded({ catalogDetail: catalogDetail as unknown as CatalogApi["catalogDetail"] });

        const opening = store.activate("p1-a");
        expect(store.pending?.card.id).toBe("p1-a");
        expect(store.pending?.document).toBe("mods/p1-a.json");
        expect(store.detail).toBeUndefined();

        release({
            card: card("p1-a"),
            description: "d",
            authors: ["a"],
            license: "MIT",
            source: "https://github.com/a/b",
            homepage: null,
            api: 2,
            sha256: "ab",
            stale: false,
        } as never);
        await opening;

        expect(store.pending).toBeUndefined();
        expect(store.detail?.sourceLabel).toBe("a/b");
        expect(store.detail?.registryName).toBe("R1");
    });

    it("drops a layout that comes late from a source the player has left", async () => {
        let release: (view: CatalogLayoutView) => void = () => {};
        const late = new Promise<CatalogLayoutView>((resolve) => (release = resolve));
        const catalogLayout = vi
            .fn()
            .mockResolvedValueOnce(layout("r1", 60))
            .mockReturnValueOnce(late)
            .mockResolvedValueOnce(layout("r1", 60));
        const store = await loaded({ catalogLayout });

        const slow = store.chooseSource("r2");
        await store.chooseSource("r1");
        release(layout("r2", 5));
        await slow;

        expect(store.layout?.registryId).toBe("r1");
        expect(store.source).toBe("r1");
    });

    it("opens the URL screen from the source list and shows an offline registry without a fault", async () => {
        const navigate = vi.fn();
        const store = await loaded(
            {
                registriesRefresh: vi.fn(async () => [registry("r1"), registry("r2", { status: "offline", modCount: null })]),
                catalogLayout: vi.fn(async (_n: string, id: string) => {
                    if (id === "r2") throw new Error("offline");
                    return layout(id, 10);
                }),
            },
            navigate,
        );

        await store.chooseSource("r2");
        expect(store.fault).toBeNull();
        expect(store.sources.find((source) => source.id === "r2")?.status).toBe("offline");

        await store.chooseSource("url");
        expect(navigate).toHaveBeenCalledWith("/instances/play/add-url");
    });
});
