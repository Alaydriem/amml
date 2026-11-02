import type { Snippet } from "svelte";

export interface PanelProps {
    /** Mono caps header text. It also names the panel. */
    heading: string;
    /** `roomy` is the taller header of the setup checks (13 px) against the settings cards (10 px). */
    roomy?: boolean;
    /** Trailing header content: a link-like button or a mono note. */
    aside?: Snippet;
    /** The rows. Use `PanelRow`, `KeyValueRow` or any block. */
    children: Snippet;
}
