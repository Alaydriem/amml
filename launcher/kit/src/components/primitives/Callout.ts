import type { Snippet } from "svelte";

export interface CalloutProps {
    /** `ok` is the "put back" note of a failed install (artboard 05b). */
    tone?: "brand" | "warn" | "ok";
    /**
     * Ground behind the text. `tint` (brand default) is the brand wash with square left corners, `raised` (warn and
     * ok default) the raised panel colour, `base` the page colour (inside a dialog).
     */
    ground?: "tint" | "raised" | "base";
    /** Wrap emphasis in `<strong>`. */
    children: Snippet;
}

export class CalloutRules {
    static ground(tone: NonNullable<CalloutProps["tone"]>, ground: CalloutProps["ground"]): "tint" | "raised" | "base" {
        return ground ?? (tone === "brand" ? "tint" : "raised");
    }
}
