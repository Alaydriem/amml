import type { Snippet } from "svelte";
import type { Severity } from "../../model";

/** Pill tones use the model's severity names; `info` draws as `brand`. */
export type PillTone = Extract<Severity, "brand" | "ok" | "warn" | "muted" | "fault" | "info">;

export type PillSize = "sm" | "md";

export interface PillProps {
    tone?: PillTone;
    /** Leading 7 px square dot in the tone colour. */
    dot?: boolean;
    /** Tinted fill (the "2 updates" pill on the instance list). Without it the pill is an outline. */
    tint?: boolean;
    /** `md` 14 px text, `sm` 13 px text (the Update pill in the mod list). */
    size?: PillSize;
    /** Makes the pill a button. */
    onclick?: () => void;
    children: Snippet;
}
