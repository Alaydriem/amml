import type { Swatch } from "./Swatch";

/** State of a catalogue row against the instance the user is adding to. */
export type ModCardState = "available" | "installed" | "update" | "needsAmml";

/** One row of a registry index page (listing spec, Part 3), with the instance state added by the app. */
export interface ModCard {
    id: string;
    registryId: string;
    title: string;
    summary: string;
    tags: string[];
    /** Newest version for the instance's api. */
    version: string;
    /** Download size in bytes of that version's zip. */
    size: number;
    /** Icon URL from the registry's `icons/`; when absent the tile shows `swatch`. */
    icon?: string;
    swatch: Swatch;
    state: ModCardState;
    /** Why the row cannot be installed, for example "Needs AMML 26.62.0". The row shows greyed and has no install. */
    blockedNote?: string;
    /** Note on a row that stays usable, for example "Later release needs AMML 26.62.0". */
    laterNote?: string;
}
