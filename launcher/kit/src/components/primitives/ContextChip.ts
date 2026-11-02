import type { TextTone } from "../../model";

/** One label and value pair of the chip ("Game" and "1.26.60.5"). */
export interface ContextChipPart {
    /** Stable key of the pair. */
    id: string;
    label: string;
    value: string;
    /** Colour of the value. Default `dim`. */
    tone?: TextTone;
}

export interface ContextChipText {
    /** Drawn between two pairs ("·"). */
    separator: string;
}

export interface ContextChipProps {
    parts: ContextChipPart[];
    text: ContextChipText;
}
