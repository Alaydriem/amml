/** What the list needs from an observer; tests pass their own. */
export interface SentinelObserver {
    observe(element: Element): void;
    unobserve(element: Element): void;
    disconnect(): void;
}
