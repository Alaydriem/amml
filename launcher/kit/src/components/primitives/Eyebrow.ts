import type { Snippet } from "svelte";

export interface EyebrowProps {
    /** `sm` 12 px, `md` 13 px, `lg` 14 px (page eyebrow). */
    size?: "sm" | "md" | "lg";
    tone?: "faint" | "warn";
    /** Element id, so a control can name itself from this caption with `labelledby`. */
    id?: string;
    children: Snippet;
}
