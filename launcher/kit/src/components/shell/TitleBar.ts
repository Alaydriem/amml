import type { Severity } from "../../model";

export interface TitleBarText {
    /** Wordmark next to the brand mark. Drawn in caps. */
    wordmark: string;
    /** Accessible name of the refresh button. */
    refresh: string;
    /** Accessible name of the settings button. */
    settings: string;
    /** Template with `{current}` and `{total}` slots for the setup caption. */
    step: string;
    /** Template with a `{fetched}` slot for the registry status. */
    registry: string;
}

export interface TitleBarProps {
    text: TitleBarText;
    /** Setup steps. When set, the right side shows the step indicator and nothing else. */
    steps?: { total: number; current: number };
    /** When the registry was fetched, as the app words it ("4 min ago"). Without it no registry status shows. */
    registryFetched?: string;
    registryTone?: Severity;
    onrefresh?: () => void;
    onsettings?: () => void;
}
