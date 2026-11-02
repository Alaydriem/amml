import { describe, expect, it } from "vitest";
import { PageRequests } from "../../../src/components/catalog/PageRequests";
import type { PageRef, ResultGroup } from "../../../src/model";
import { CatalogFixtures } from "../../../src/fixtures/catalog";

const page3: PageRef = { registryId: "community", page: 3 };

// Rows 40 to 59 loaded of 100, 20 to a page: page 3 is loaded, pages 4 and 5 are missing.
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

const page4: PageRef = { registryId: "community", page: 4 };

describe("PageRequests", () => {
    it("gives a page out once, and not while the app loads it", () => {
        const requests = new PageRequests();

        expect(requests.claim(page4, [])).toBe(true);
        expect(requests.claim(page4, [])).toBe(false);
        expect(requests.claim(page3, [page3])).toBe(false);
        expect(requests.claim({ registryId: "official", page: 4 }, [page4])).toBe(true);
    });

    it("releases a page when its rows arrived", () => {
        const requests = new PageRequests();
        requests.claim(page4, []);
        const arrived = group({ rows: [...CatalogFixtures.communityRows, ...CatalogFixtures.generated("community", 60, 80)] });

        requests.sync([], [arrived]);

        expect(requests.claim(page4, [])).toBe(true);
    });

    it("releases a page that left loading without its rows", () => {
        const requests = new PageRequests();
        requests.claim(page4, []);
        requests.sync([page4], [group()]);
        expect(requests.claim(page4, [page4])).toBe(false);

        requests.sync([], [group()]);

        expect(requests.claim(page4, [])).toBe(true);
    });

    it("keeps a page the app never reported as loading until its rows arrive", () => {
        const requests = new PageRequests();
        requests.claim(page4, []);

        requests.sync([], [group()]);

        expect(requests.claim(page4, [])).toBe(false);
    });

    it("forgets every page on reset", () => {
        const requests = new PageRequests();
        requests.claim(page4, []);
        requests.claim({ registryId: "community", page: 5 }, []);

        requests.reset();

        expect(requests.claim(page4, [])).toBe(true);
        expect(requests.claim({ registryId: "community", page: 5 }, [])).toBe(true);
    });
});
