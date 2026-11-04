import type { Severity } from "./Severity";

/** One line of the "This PC" check list (artboard 01) or the URL install checks (06d). */
export interface SystemCheck {
    id: string;
    title: string;
    detail?: string;
    /** `ok` ticked, `warn` exclamation mark, `muted` empty box. */
    severity: Extract<Severity, "ok" | "warn" | "muted">;
    /** Right-hand text, for example "Found", "212 GB free", "api 2". */
    value?: string;
    /** When set, the row shows a button with this label instead of `value`. */
    actionLabel?: string;
    /** The action needs admin rights (shield icon). */
    elevated?: boolean;
}
