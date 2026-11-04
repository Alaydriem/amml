import type { GameVersion } from "../../model";
import { Template } from "../../text/Template";
import { Units } from "../../text/Units";
import type { PillTone } from "../primitives/Pill";

export interface VersionRowText {
    /** Second line of an installed row ("Already in the version store · local copy only"). */
    installedNote: string;
    installed: string;
    /** Template with a `{size}` slot, for example "~{size} download". */
    download: string;
    unsupported: string;
}

export interface VersionRowProps {
    version: GameVersion;
    text: VersionRowText;
    selected?: boolean;
    /** Whether the row is the tab stop of its group. */
    stop?: boolean;
    /** Called with the version string when the row is chosen. Never called for an unsupported row. */
    onselect?: (version: string) => void;
    /** Key presses on the row, for the group's arrow-key movement. */
    onkeydown?: (event: KeyboardEvent) => void;
}

export class VersionRowRules {
    /** Tone and text of the state pill. */
    static pill(version: GameVersion, text: VersionRowText): { tone: PillTone; label: string } {
        switch (version.state) {
            case "installed":
                return { tone: "ok", label: text.installed };
            case "download":
                return {
                    tone: "brand",
                    label: Template.fill(text.download, { size: Units.en.size(version.size ?? 0, { digits: 1 }) }),
                };
            case "unsupported":
                return { tone: "muted", label: text.unsupported };
        }
    }
}
