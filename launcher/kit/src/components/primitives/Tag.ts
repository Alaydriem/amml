import type { Snippet } from "svelte";
import type { TextTone } from "../../model";

export interface TagProps {
    /** Colour of the text, and of the edge when `bordered`. Default `faint`. */
    tone?: Extract<TextTone, "faint" | "ok" | "warn" | "fault" | "brand">;
    /** Draws a 1 px edge around the tag (the Core tag). Default true. */
    bordered?: boolean;
    /** The tag text ("Core", "Built in"). Drawn in caps. */
    children: Snippet;
}
