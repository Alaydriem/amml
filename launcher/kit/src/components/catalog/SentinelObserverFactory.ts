import type { SentinelCallback } from "./SentinelCallback";
import type { SentinelObserver } from "./SentinelObserver";

/**
 * Makes an observer for the scroll container. `marginPx` is how far beyond the container, above and below, an
 * element counts as in view.
 */
export type SentinelObserverFactory = (
    root: Element | null,
    onenter: SentinelCallback,
    marginPx: number,
) => SentinelObserver;
