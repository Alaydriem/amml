import type { InstalledMod } from "../../model";
import type { MenuItem } from "../primitives/Menu";
import { ModTableRowRules, type ModTableRowText } from "./ModTableRow";

export interface ModTableText {
    /** The mono caps line above the table, for example "Core · 2 · Mods · 6". */
    sectionLabel: string;
    /** The Update all button text, with the count: "Update all (2)". */
    updateAll: string;
    addMod: string;
    /** Column heads. `state` and `actions` are read by a screen reader only. */
    columns: { on: string; name: string; installed: string; available: string; state: string; actions: string };
    /** Accessible name of the table. */
    tableLabel: string;
    row: ModTableRowText;
}

export interface ModTableProps {
    mods: InstalledMod[];
    text: ModTableText;
    /** Items of each non-core row menu. */
    menuItems: MenuItem[];
    onupdateall?: () => void;
    onadd?: () => void;
    ontoggle?: (id: string, enabled: boolean) => void;
    onmenu?: (id: string, itemId: string) => void;
    onupdate?: (id: string) => void;
}

export class ModTableRules {
    /**
     * Number of rows whose state is `update`; Update all is disabled at zero. A mod from a URL never counts, and
     * neither does a local AMML build: Update all does not replace it, only its own Update pill does.
     */
    static updateCount(mods: readonly InstalledMod[]): number {
        return mods.filter((mod) => !mod.localBuild && ModTableRowRules.state(mod) === "update").length;
    }
}
