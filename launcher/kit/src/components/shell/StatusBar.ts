import type { Severity } from "../../model";

export interface StatusBarText {
    /** Template for the left text, with `{version}` and `{dataPath}` slots. */
    left: string;
    /** Template for the store status, with a `{version}` slot. */
    store: string;
}

export interface StatusBarProps {
    text: StatusBarText;
    /** Launcher version. */
    version: string;
    /** Folder the launcher keeps its data in, as shown. */
    dataPath: string;
    /** Replaces the left text for a moment (a copy confirmation). */
    message?: string;
    /** Version of the registered Store game. Without it no store status shows. */
    storeVersion?: string;
    storeTone?: Severity;
}
