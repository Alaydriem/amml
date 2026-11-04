import type { Snippet } from "svelte";
import type { InstalledMod } from "../model";
import type { ModTableText } from "../components/mods/ModTable";
import { ModTableRules } from "../components/mods/ModTable";
import type { ModTableRowText } from "../components/mods/ModTableRow";
import type { MenuItem } from "../components/primitives/Menu";
import { Template } from "../text/Template";

export interface InstanceDetailScreenText {
    /** Template with `{core}` and `{mods}` slots: "Core · {core} · Mods · {mods}". `mods` counts every mod. */
    sectionLabel: string;
    /** Template with a `{count}` slot: "Update all ({count})". */
    updateAll: string;
    addMod: string;
    columns: ModTableText["columns"];
    tableLabel: string;
    row: ModTableRowText;
    /** Items of the menu of every non-core row. */
    menu: MenuItem[];
}

export interface InstanceDetailScreenProps {
    mods: InstalledMod[];
    text: InstanceDetailScreenText;
    /** The back link and the instance header (`InstanceHeader`). */
    header: Snippet;
    onupdateall?: () => void;
    onadd?: () => void;
    ontoggle?: (id: string, enabled: boolean) => void;
    onmenu?: (id: string, itemId: string) => void;
    onupdate?: (id: string) => void;
}

export class InstanceDetailScreenRules {
    /** The table text with the counts filled in. */
    static tableText(text: InstanceDetailScreenText, mods: readonly InstalledMod[]): ModTableText {
        const core = mods.filter((mod) => mod.origin === "core").length;
        return {
            sectionLabel: Template.fill(text.sectionLabel, { core, mods: mods.length }),
            updateAll: Template.fill(text.updateAll, { count: ModTableRules.updateCount(mods) }),
            addMod: text.addMod,
            columns: text.columns,
            tableLabel: text.tableLabel,
            row: text.row,
        };
    }
}
