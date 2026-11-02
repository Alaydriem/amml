import type { ModCard } from "./ModCard";

/**
 * One block of the mod result list. Browsing one registry gives one group per letter; searching every registry
 * gives one group per registry.
 *
 * Positions count in the registry's whole index for one api, in the order the registry lists it. A group covers the
 * index range `start` to `start + total` (not included). Its loaded `rows` begin at index `offset`. The list draws
 * the rows from `start` to `offset` and from `offset + rows.length` to the end of the range as a spacer for each
 * index page they touch, and asks the app for each page that comes into view.
 */
export interface ResultGroup {
    id: string;
    label: string;
    /** Registry the rows come from; the paging callbacks name it. */
    registryId: string;
    /** Rows loaded so far, in order, starting at index `offset`. */
    rows: ModCard[];
    /** Registry index of `rows[0]`. */
    offset: number;
    /** Registry index of the group's first row, loaded or not. Default 0. */
    start?: number;
    /** Rows in the whole group, loaded or not. */
    total: number;
    /** Rows per index page (100 for schema 2). */
    pageSize: number;
    /** When set, the group stops after `rows` and shows a "+N more" row instead of paging. */
    collapsedMore?: number;
}
