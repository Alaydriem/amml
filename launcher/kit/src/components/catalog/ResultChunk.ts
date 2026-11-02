/**
 * A run of rows of one group that is not loaded yet, clipped to one registry index page. `from` and `to` are
 * registry index positions; `from` is included, `to` is not.
 */
export interface ResultChunk {
    /** `before` the loaded rows or `after` them. */
    side: "before" | "after";
    /** Registry index page, counted from 1. */
    page: number;
    from: number;
    to: number;
}
