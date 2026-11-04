import type { TextTone } from "../../model";

export interface KeyValueRowProps {
    label: string;
    value: string;
    /** Colour of the value. Default `strong`; `brand` is the colour of a source or link. */
    tone?: TextTone;
    /** Mono value (default true, as in the detail pane). */
    mono?: boolean;
    /** Makes the value a button that calls this. */
    onopen?: () => void;
}
