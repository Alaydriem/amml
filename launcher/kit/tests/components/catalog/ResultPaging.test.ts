import { describe, expect, it } from "vitest";
import { ResultPaging } from "../../../src/components/catalog/ResultPaging";
import type { ResultGroup } from "../../../src/model";
import { CatalogFixtures } from "../../../src/fixtures/catalog";

// 100 rows, 20 to a page, rows 40 to 59 loaded: pages 1 and 2 before, pages 4 and 5 after.
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

function runs(chunks: ReturnType<typeof ResultPaging.chunks>) {
    return chunks.map((chunk) => [chunk.side, chunk.page, chunk.from, chunk.to]);
}

describe("ResultPaging.page", () => {
    it("counts registry index pages from 1 at the page edges", () => {
        expect(ResultPaging.page(0, 100)).toBe(1);
        expect(ResultPaging.page(99, 100)).toBe(1);
        expect(ResultPaging.page(100, 100)).toBe(2);
        expect(ResultPaging.page(1599, 100)).toBe(16);
    });
});

describe("ResultPaging.chunks", () => {
    it("lists the unloaded runs before and after the loaded rows, one page each", () => {
        expect(runs(ResultPaging.chunks(group()))).toEqual([
            ["before", 1, 0, 20],
            ["before", 2, 20, 40],
            ["after", 4, 60, 80],
            ["after", 5, 80, 100],
        ]);
    });

    it("clips a run to the loaded range when the loaded rows do not start on a page edge", () => {
        const chunks = ResultPaging.chunks(group({ offset: 30, rows: CatalogFixtures.communityRows.slice(0, 15), total: 70 }));

        expect(runs(chunks)).toEqual([
            ["before", 1, 0, 20],
            ["before", 2, 20, 30],
            ["after", 3, 45, 60],
            ["after", 4, 60, 70],
        ]);
    });

    it("counts positions in the whole registry index for a group that starts in the middle of it", () => {
        const chunks = ResultPaging.chunks(group({ start: 100, offset: 120, total: 60, rows: CatalogFixtures.communityRows }));

        expect(runs(chunks)).toEqual([
            ["before", 6, 100, 120],
            ["after", 8, 140, 160],
        ]);
    });

    it("treats a group with no loaded rows as one run from its start", () => {
        const chunks = ResultPaging.chunks(group({ start: 30, offset: 30, total: 30, rows: [] }));

        expect(runs(chunks)).toEqual([
            ["after", 2, 30, 40],
            ["after", 3, 40, 60],
        ]);
    });

    it("has no run when every row is loaded or when the group shows a more row", () => {
        expect(ResultPaging.chunks(group({ offset: 0, total: 20 }))).toEqual([]);
        expect(ResultPaging.chunks(group({ collapsedMore: 80 }))).toEqual([]);
    });
});

describe("ResultPaging.missingKeys", () => {
    it("names a page shared by two groups of one registry once", () => {
        const a = group({ id: "a", start: 0, offset: 0, total: 30, rows: [] });
        const b = group({ id: "b", start: 30, offset: 30, total: 30, rows: [] });

        expect([...ResultPaging.missingKeys([a, b])]).toEqual(["community:1", "community:2", "community:3"]);
    });

    it("keeps the pages of two registries apart", () => {
        const a = group({ id: "a", offset: 0, total: 20, rows: [] });
        const b = group({ id: "b", registryId: "official", offset: 0, total: 20, rows: [] });

        expect([...ResultPaging.missingKeys([a, b])]).toEqual(["community:1", "official:1"]);
    });
});

describe("ResultPaging.lookAhead", () => {
    it("is one page of the tallest group, in px", () => {
        expect(ResultPaging.lookAhead([group({ pageSize: 20 }), group({ pageSize: 100 })])).toBe(100 * ResultPaging.ROW_HEIGHT);
        expect(ResultPaging.lookAhead([])).toBe(0);
    });
});
