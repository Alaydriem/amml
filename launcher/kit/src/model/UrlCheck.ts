import type { SystemCheck } from "./SystemCheck";
import type { TextTone } from "./TextTone";

/** One line of the URL install checks. `valueTone` colours the right-hand value (`api 2` green, `no checksum` amber). */
export interface UrlCheck extends SystemCheck {
    valueTone?: Extract<TextTone, "faint" | "dim" | "ok" | "warn">;
}
