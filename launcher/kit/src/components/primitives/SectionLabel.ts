import type { Snippet } from "svelte";

export interface SectionLabelProps {
    /** Sticks to the top of the scrolling list. */
    sticky?: boolean;
    children: Snippet;
}
