import type { Snippet } from "svelte";
import type { Severity } from "../../model";

export interface StatusDotProps {
    tone?: Severity;
    /** `md` is an 8 px dot with 15 px text (status bar); `sm` a 7 px dot with 14 px text (row state). */
    size?: "sm" | "md";
    /** Mono caps text (the registry status in the title bar). */
    caps?: boolean;
    children: Snippet;
}
