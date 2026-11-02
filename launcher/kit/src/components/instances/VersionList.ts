import type { GameVersion } from "../../model";
import type { VersionRowText } from "./VersionRow";

export interface VersionListProps {
    versions: GameVersion[];
    text: VersionRowText;
    /** Accessible name of the list. */
    label: string;
    /** Version string of the selected row. The list never changes it: the parent answers `onchange`. */
    value?: string;
    /** Called with the version the user asks for, by click or arrow key. Never called for an unsupported version. */
    onchange?: (version: string) => void;
}

export class VersionListRules {
    /** True for a row the arrow keys skip. */
    static disabled(version: GameVersion | undefined): boolean {
        return version === undefined || version.state === "unsupported";
    }

    /** The row that holds the tab stop: the selected one, else the first supported one. */
    static stop(versions: GameVersion[], selected: number): number {
        if (selected >= 0) return selected;
        const first = versions.findIndex((version) => !VersionListRules.disabled(version));
        return first < 0 ? 0 : first;
    }
}
