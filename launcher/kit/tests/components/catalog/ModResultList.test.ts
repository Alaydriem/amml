import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import ModResultList from "../../../src/components/catalog/ModResultList.svelte";
import { ModResultRowRules } from "../../../src/components/catalog/ModResultRow";
import { ResultPaging } from "../../../src/components/catalog/ResultPaging";
import type { PageRef, ResultGroup } from "../../../src/model";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModBrowseEn } from "../../../src/text/en/addModBrowse";
import { FakeObserver } from "./FakeObserver";

const text = addModBrowseEn.list;

// 100 rows, 20 to a page, rows 40 to 59 loaded (page 3): pages 1 and 2 before it, pages 4 and 5 after it.
function group(overrides: Partial<ResultGroup> = {}): ResultGroup {
    return {
        id: "g",
        label: "C",
        registryId: "community",
        rows: CatalogFixtures.communityRows,
        offset: 40,
        total: 100,
        pageSize: 20,
        ...overrides,
    };
}

function withPage4(): ResultGroup {
    return group({ rows: [...CatalogFixtures.communityRows, ...CatalogFixtures.generated("community", 60, 80)] });
}

const page = (n: number): PageRef => ({ registryId: "community", page: n });

describe("ModResultRowRules.toggle", () => {
    it("adds an id once and removes it on clear", () => {
        expect(ModResultRowRules.toggle(["a"], "b", true)).toEqual(["a", "b"]);
        expect(ModResultRowRules.toggle(["a", "b"], "b", true)).toEqual(["a", "b"]);
        expect(ModResultRowRules.toggle(["a", "b"], "a", false)).toEqual(["b"]);
    });
});

describe("ModResultList paging", () => {
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it("calls onloadmore once for a page that comes into view", () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        render(ModResultList, { groups: [group()], text, observerFactory: fake.factory, onloadmore });

        fake.enter(4);
        fake.enter(4);

        expect(onloadmore).toHaveBeenCalledExactlyOnceWith(page(4));
    });

    it("does not call for a page the app is loading", () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        render(ModResultList, { groups: [group()], text, observerFactory: fake.factory, loading: [page(5)], onloadmore });

        fake.enter(5);

        expect(onloadmore).not.toHaveBeenCalled();
    });

    it("has no spacer for the loaded page", () => {
        const fake = new FakeObserver();
        render(ModResultList, { groups: [group()], text, observerFactory: fake.factory });

        expect(fake.sentinels(3)).toEqual([]);
        expect(fake.sentinels(2)).toHaveLength(1);
    });

    it("asks again for a page that arrived and went missing again, in the same generation", async () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        const { rerender } = render(ModResultList, { groups: [group()], text, observerFactory: fake.factory, onloadmore });

        fake.enter(4);
        await rerender({ groups: [withPage4()] });
        await rerender({ groups: [group()] });
        fake.enter(4);

        expect(onloadmore.mock.calls.map(([ref]) => ref)).toEqual([page(4), page(4)]);
    });

    it("asks again for a page that left loading without its rows", async () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        const { rerender } = render(ModResultList, { groups: [group()], text, observerFactory: fake.factory, onloadmore });

        fake.enter(4);
        await rerender({ loading: [page(4)] });
        fake.enter(4);
        expect(onloadmore).toHaveBeenCalledTimes(1);

        await rerender({ loading: [] });
        fake.enter(4);

        expect(onloadmore.mock.calls.map(([ref]) => ref)).toEqual([page(4), page(4)]);
    });

    it("asks again after a new generation, with new elements to observe", async () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        const { rerender } = render(ModResultList, { groups: [group()], text, observerFactory: fake.factory, onloadmore });
        const before = fake.sentinel(4);
        fake.enter(4);

        await rerender({ groups: [group({ rows: CatalogFixtures.generated("community", 40, 60) })], generation: 1 });
        const after = fake.sentinel(4);
        fake.onenter(after);

        expect(after).not.toBe(before);
        expect(fake.observed.has(before)).toBe(false);
        expect(onloadmore.mock.calls.map(([ref]) => ref)).toEqual([page(4), page(4)]);
    });

    it("asks once when two groups of one registry need the same page", () => {
        const fake = new FakeObserver();
        const onloadmore = vi.fn();
        const a = group({ id: "a", label: "A", start: 0, offset: 0, total: 30, rows: [] });
        const b = group({ id: "b", label: "B", start: 30, offset: 30, total: 30, rows: [] });
        render(ModResultList, { groups: [a, b], text, observerFactory: fake.factory, onloadmore });

        expect(fake.sentinels(2)).toHaveLength(2);
        fake.enter(2, 0);
        fake.enter(2, 1);

        expect(onloadmore).toHaveBeenCalledExactlyOnceWith(page(2));
    });

    it("draws one spacer for each missing page, not one row for each missing row", () => {
        const fake = new FakeObserver();
        const big = group({ total: 1412 });
        const { container } = render(ModResultList, { groups: [big], text, observerFactory: fake.factory });

        const chunks = ResultPaging.chunks(big);
        const spacers = container.querySelectorAll<HTMLElement>(".amml-results__spacer");

        expect(chunks).toHaveLength(70);
        expect(spacers).toHaveLength(70);
        expect(container.querySelectorAll(".amml-loading-row")).toHaveLength(70);
        expect(spacers[0]?.style.height).toBe(`${20 * ResultPaging.ROW_HEIGHT}px`);
        expect(container.querySelectorAll("li")).toHaveLength(70 + CatalogFixtures.communityRows.length);
        expect(screen.getByText("loading 61–80")).toBeInTheDocument();
    });

    it("looks one page ahead of the visible rows", () => {
        const fake = new FakeObserver();
        render(ModResultList, { groups: [group({ pageSize: 100 })], text, observerFactory: fake.factory });

        expect(fake.marginPx).toBe(100 * ResultPaging.ROW_HEIGHT);
    });

    it("scrolls to the first loaded row once the rows arrive, and once for each generation", async () => {
        vi.spyOn(HTMLElement.prototype, "offsetTop", "get").mockReturnValue(2280);
        vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(29);
        const fake = new FakeObserver();
        const { container, rerender } = render(ModResultList, {
            groups: [group({ rows: [] })],
            text,
            observerFactory: fake.factory,
        });
        const list = container.querySelector<HTMLElement>(".amml-results")!;
        expect(list.scrollTop).toBe(0);

        await rerender({ groups: [group()] });
        expect(list.scrollTop).toBe(2280 - 29);

        list.scrollTop = 5;
        await rerender({ groups: [withPage4()] });
        expect(list.scrollTop).toBe(5);

        await rerender({ groups: [group()], generation: 1 });
        expect(list.scrollTop).toBe(2280 - 29);
    });
});

describe("ModResultList structure", () => {
    it("draws one list for each group, with a heading that names it and a list item for each row", () => {
        const fake = new FakeObserver();
        const groups = CatalogFixtures.searchGroups();
        render(ModResultList, { groups, text, observerFactory: fake.factory, showGroupCount: true });

        const lists = screen.getAllByRole("list");
        expect(lists).toHaveLength(3);
        const community = screen.getByRole("region", { name: "Community index · 27" });
        expect(within(community).getAllByRole("listitem").length).toBeGreaterThanOrEqual(5);
        expect(screen.getAllByRole("heading").map((heading) => heading.textContent?.trim())).toEqual([
            "Official · 1",
            "Community index · 27",
            "Friends' server pack · 3",
        ]);
        expect(screen.getAllByRole("listitem").length).toBeGreaterThanOrEqual(8);
    });

    it("marks the list busy and says so once while pages load", async () => {
        const fake = new FakeObserver();
        const { rerender } = render(ModResultList, { groups: [group()], text, observerFactory: fake.factory });
        expect(screen.getByRole("region", { name: "Mods" })).toHaveAttribute("aria-busy", "false");
        expect(screen.getByRole("status")).toHaveTextContent("");

        await rerender({ loading: [page(4), page(5)] });

        expect(screen.getByRole("region", { name: "Mods" })).toHaveAttribute("aria-busy", "true");
        expect(screen.getAllByRole("status")).toHaveLength(1);
        expect(screen.getByRole("status")).toHaveTextContent("Loading more mods");
    });

    it("shows a more row that calls onshowmore and does not page", async () => {
        const fake = new FakeObserver();
        const onshowmore = vi.fn();
        const onloadmore = vi.fn();
        render(ModResultList, {
            groups: [group({ id: "community", offset: 0, collapsedMore: 23, total: 27, rows: CatalogFixtures.communityRows.slice(0, 4) })],
            text,
            observerFactory: fake.factory,
            onshowmore,
            onloadmore,
        });

        await fireEvent.click(screen.getByRole("button", { name: "+23 more in C" }));

        expect(onshowmore).toHaveBeenCalledExactlyOnceWith("community");
        expect(fake.observed.size).toBe(0);
        expect(onloadmore).not.toHaveBeenCalled();
    });
});

describe("ModResultList selection", () => {
    const rows = () => group({ offset: 0, total: 20 });

    it("asks for the new ticked ids and keeps its own state out of it", async () => {
        const fake = new FakeObserver();
        const onchange = vi.fn();
        const onactivate = vi.fn();
        render(ModResultList, {
            groups: [rows()],
            checked: ["@kv/chunkborders"],
            text,
            observerFactory: fake.factory,
            onchange,
            onactivate,
        });
        const chatHeads = screen.getByRole("checkbox", { name: "Select Chat Heads" });

        await fireEvent.click(chatHeads);

        expect(onchange).toHaveBeenCalledExactlyOnceWith(["@kv/chunkborders", "@mira/chatheads"]);
        expect(chatHeads).toHaveAttribute("aria-checked", "false");
        expect(screen.getByRole("checkbox", { name: "Select Chunk Borders" })).toHaveAttribute("aria-checked", "true");
        expect(onactivate).not.toHaveBeenCalled();
    });

    it("cannot tick an installed row", () => {
        const fake = new FakeObserver();
        render(ModResultList, { groups: [rows()], text, observerFactory: fake.factory });

        expect(screen.getByRole("checkbox", { name: "Select Clock HUD" })).toBeDisabled();
        expect(screen.getByText("Installed")).toBeInTheDocument();
    });
});
