import type { SystemCheck } from "../../model";
import type { StatusMarkState } from "../primitives/StatusMark";

export interface CheckRowText {
    /** Read by assistive technology for a passed check ("Passed"). */
    ok: string;
    /** Read for a check that needs attention ("Needs attention"). */
    warn: string;
    /** Read for an optional check that is not set up ("Optional"). */
    muted: string;
}

export interface CheckRowProps {
    check: SystemCheck;
    text: CheckRowText;
    /** Called with the check id when the action button is pressed. */
    onaction?: (id: string) => void;
}

export class CheckRowRules {
    /** Values that carry a number or a path ("212 GB free", "api 2") are mono; words ("Found") are not. */
    static isMonoValue(value: string | undefined): boolean {
        return value !== undefined && /\d/.test(value);
    }

    /** The mark state of a check: `muted` draws the empty box. */
    static mark(severity: SystemCheck["severity"]): StatusMarkState {
        return severity === "muted" ? "pending" : severity;
    }
}
