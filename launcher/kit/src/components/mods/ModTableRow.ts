import type { InstalledMod } from "../../model";
import { Template } from "../../text/Template";
import type { MenuItem } from "../primitives/Menu";
import type { ModRowControls } from "./ModRowControls";

/** Which cell the right-hand state column of a row shows. */
export type ModRowState = "current" | "update" | "unofficial";

/** Text of one row. `toggleLabel` and `menuLabel` are templates with a `{title}` slot. */
export interface ModTableRowText {
    coreTag: string;
    upToDate: string;
    update: string;
    unofficial: string;
    /** Shown in the available column when no registry lists the mod. */
    noVersion: string;
    toggleLabel: string;
    menuLabel: string;
    /** Template with `{id}` and `{note}` slots: "{id} · {note}". Used when the mod has a note. */
    subline: string;
}

export interface ModTableRowProps {
    mod: InstalledMod;
    text: ModTableRowText;
    /** Items of the row menu; a core mod has no menu. */
    menuItems: MenuItem[];
    ontoggle?: (id: string, enabled: boolean) => void;
    onmenu?: (id: string, itemId: string) => void;
    onupdate?: (id: string) => void;
}

/** A version split into its numeric core and its pre-release identifiers. */
interface ParsedVersion {
    core: number[];
    pre: string[];
}

export class ModTableRowRules {
    private static readonly VERSION = /^v?(\d+(?:\.\d+)*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/;

    /**
     * True when `available` is newer than `installed`. The numeric cores compare part by part (a missing part is
     * 0). A pre-release ranks below its release (`1.2.0-beta` < `1.2.0`); two pre-releases compare identifier by
     * identifier, numbers by value. A version that cannot be read is never newer.
     */
    static isNewer(available: string, installed: string): boolean {
        const a = ModTableRowRules.parse(available);
        const b = ModTableRowRules.parse(installed);
        if (!a || !b) return false;
        const length = Math.max(a.core.length, b.core.length);
        for (let i = 0; i < length; i++) {
            const x = a.core[i] ?? 0;
            const y = b.core[i] ?? 0;
            if (x !== y) return x > y;
        }
        return ModTableRowRules.comparePre(a.pre, b.pre) > 0;
    }

    /** True when a registry lists a newer version than the installed one. */
    static hasUpdate(mod: InstalledMod): boolean {
        return mod.availableVersion !== undefined && ModTableRowRules.isNewer(mod.availableVersion, mod.installedVersion);
    }

    /** A mod from a URL is unofficial and gets no update check. Otherwise the row shows an update or none. */
    static state(mod: InstalledMod): ModRowState {
        if (mod.origin === "url") return "unofficial";
        return ModTableRowRules.hasUpdate(mod) ? "update" : "current";
    }

    /** A core mod shows the tag and has no toggle and no menu; any other mod has both. */
    static controls(mod: InstalledMod): ModRowControls {
        const core = mod.origin === "core";
        return { toggle: !core, menu: !core, coreTag: core };
    }

    /** The id with its note after it, or the id alone. */
    static subline(mod: InstalledMod, text: ModTableRowText): string {
        return mod.note ? Template.fill(text.subline, { id: mod.id, note: mod.note }) : mod.id;
    }

    private static parse(version: string): ParsedVersion | undefined {
        const match = ModTableRowRules.VERSION.exec(version.trim());
        if (!match) return undefined;
        return {
            core: (match[1] ?? "").split(".").map((part) => Number(part)),
            pre: match[2] === undefined ? [] : match[2].split("."),
        };
    }

    /** Positive when `a` ranks above `b`. No pre-release identifiers rank above any. */
    private static comparePre(a: readonly string[], b: readonly string[]): number {
        if (a.length === 0 && b.length === 0) return 0;
        if (a.length === 0) return 1;
        if (b.length === 0) return -1;
        for (let i = 0; i < Math.max(a.length, b.length); i++) {
            const x = a[i];
            const y = b[i];
            if (x === undefined) return -1;
            if (y === undefined) return 1;
            const numeric = /^\d+$/.test(x) && /^\d+$/.test(y);
            if (numeric && Number(x) !== Number(y)) return Number(x) > Number(y) ? 1 : -1;
            if (!numeric && x !== y) return x > y ? 1 : -1;
        }
        return 0;
    }
}
