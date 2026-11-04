import type { Snippet } from "svelte";

export interface PanelRowProps {
    /** `md` 14 px padding with a 22 px gap; `sm` 11 px padding (link rows). */
    size?: "md" | "sm";
    /** Leading control or mark (a `StatusMark`, a swatch). */
    leading?: Snippet;
    /** Row title, 17 px. */
    title: Snippet;
    /** Second line under the title. */
    detail?: Snippet;
    /** Trailing content: a state, a value, a control. */
    aside?: Snippet;
}
