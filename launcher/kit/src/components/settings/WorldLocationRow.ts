import type { WorldLocation } from "../../model";

export interface WorldLocationRowText {
    shared: string;
    own: string;
}

export interface WorldLocationRowProps {
    location: WorldLocation;
    text: WorldLocationRowText;
}
