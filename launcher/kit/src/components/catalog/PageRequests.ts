import type { PageRef, ResultGroup } from "../../model";
import { ResultPaging } from "./ResultPaging";

/**
 * The pages the list has asked the app for. A page is asked for once; it can be asked for again after it is
 * released. A page is released when
 * - its rows arrived (no group lacks it any more),
 * - it left `loading` without its rows (the load failed or the app dropped it), or
 * - the list is reset for a new generation.
 * A page the app never reports as loading is released only by its rows or a reset.
 */
export class PageRequests {
    private readonly requested = new Set<string>();
    private readonly seenLoading = new Set<string>();

    /** True, and remembers the page, when it is neither loading nor already asked for. */
    claim(ref: PageRef, loading: readonly PageRef[]): boolean {
        const key = ResultPaging.key(ref);
        if (this.requested.has(key)) return false;
        if (loading.some((entry) => ResultPaging.key(entry) === key)) return false;
        this.requested.add(key);
        return true;
    }

    /** Releases the pages whose rows arrived or whose load ended without rows. */
    sync(loading: readonly PageRef[], groups: readonly ResultGroup[]): void {
        const loadingKeys = new Set(loading.map((entry) => ResultPaging.key(entry)));
        for (const key of loadingKeys) {
            if (this.requested.has(key)) this.seenLoading.add(key);
        }
        const missing = ResultPaging.missingKeys(groups);
        for (const key of [...this.requested]) {
            const ended = this.seenLoading.has(key) && !loadingKeys.has(key);
            if (ended || !missing.has(key)) {
                this.requested.delete(key);
                this.seenLoading.delete(key);
            }
        }
    }

    /** Forgets every page; the app starts a new generation of rows. */
    reset(): void {
        this.requested.clear();
        this.seenLoading.clear();
    }
}
