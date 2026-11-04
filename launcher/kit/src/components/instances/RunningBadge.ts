import type { Snippet } from "svelte";

export interface RunningBadgeProps {
    /** Stretches to the parent's width. */
    block?: boolean;
    /** The status text ("Running"). */
    children: Snippet;
}
