import { readFileSync } from "node:fs";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { en } from "@amml/launcher-kit";
import { I18n, type LanguagePack } from "#lib/i18n/index.ts";
import { AboutSettingsTextSet } from "#lib/text/AboutSettingsTextSet.ts";
import { ActivityTextSet } from "#lib/text/ActivityTextSet.ts";
import { AddModBrowseTextSet } from "#lib/text/AddModBrowseTextSet.ts";
import { AddModUrlTextSet } from "#lib/text/AddModUrlTextSet.ts";
import { DialogsTextSet } from "#lib/text/DialogsTextSet.ts";
import { DownloadsSettingsTextSet } from "#lib/text/DownloadsSettingsTextSet.ts";
import { FirstRunAdoptTextSet } from "#lib/text/FirstRunAdoptTextSet.ts";
import { FirstRunChecksTextSet } from "#lib/text/FirstRunChecksTextSet.ts";
import { GameVersionsSettingsTextSet } from "#lib/text/GameVersionsSettingsTextSet.ts";
import { InstallProgressTextSet } from "#lib/text/InstallProgressTextSet.ts";
import { InstanceDetailTextSet } from "#lib/text/InstanceDetailTextSet.ts";
import { InstancesTextSet } from "#lib/text/InstancesTextSet.ts";
import { NewInstanceTextSet } from "#lib/text/NewInstanceTextSet.ts";
import { RegistriesSettingsTextSet } from "#lib/text/RegistriesSettingsTextSet.ts";
import { SharedDataSettingsTextSet } from "#lib/text/SharedDataSettingsTextSet.ts";
import { ShellTextSet } from "#lib/text/ShellTextSet.ts";
import { UnitsTextSet } from "#lib/text/UnitsTextSet.ts";

const BUILT: Record<string, () => unknown> = {
    shell: () => ShellTextSet.build(),
    units: () => UnitsTextSet.build(),
    firstRunChecks: () => FirstRunChecksTextSet.build(),
    firstRunAdopt: () => FirstRunAdoptTextSet.build(),
    instances: () => InstancesTextSet.build(),
    instanceRow: () => InstancesTextSet.instanceRow(),
    instanceMenu: () => InstancesTextSet.instanceMenu(),
    instanceHeader: () => InstancesTextSet.instanceHeader(),
    newInstance: () => NewInstanceTextSet.build(),
    installProgress: () => InstallProgressTextSet.build(),
    instanceDetail: () => InstanceDetailTextSet.build(),
    addModBrowse: () => AddModBrowseTextSet.build(),
    addModUrl: () => AddModUrlTextSet.build(),
    registriesSettings: () => RegistriesSettingsTextSet.build(),
    gameVersionsSettings: () => GameVersionsSettingsTextSet.build(),
    sharedDataSettings: () => SharedDataSettingsTextSet.build(),
    downloadsSettings: () => DownloadsSettingsTextSet.build(),
    aboutSettings: () => AboutSettingsTextSet.build(),
    dialogs: () => DialogsTextSet.build(),
};

// the pseudo pack `mise run launcher-app:i18n-build` writes from the catalog
const PSEUDO = join(process.cwd(), "src-tauri", "resources", "i18n", "en_XA.json");

// every string of a text value with the path that leads to it
function leaves(value: unknown, path = ""): [string, string][] {
    if (typeof value === "string") return [[path, value]];
    if (Array.isArray(value)) return value.flatMap((item, index) => leaves(item, `${path}[${index}]`));
    if (value !== null && typeof value === "object") {
        return Object.entries(value).flatMap(([key, item]) => leaves(item, path === "" ? key : `${path}.${key}`));
    }
    return [];
}

// ids and tones are keys for the app, not words for the player
function isWords(path: string, text: string): boolean {
    const last = path.split(".").pop() ?? "";
    if (/^(id|tone)$/.test(last) || /\.(id|tone)$/.test(path) || /\](\.id|\.tone)$/.test(path)) return false;
    return /[A-Za-z]/.test(text.replace(/\{\w+\}/g, ""));
}

describe("text sets", () => {
    afterEach(() => I18n.adopt(null));

    it("builds every kit text equal to the kit English when no pack is loaded", () => {
        for (const [key, build] of Object.entries(BUILT)) {
            expect(build(), key).toEqual((en as Record<string, unknown>)[key]);
        }
        expect(DialogsTextSet.removeVersion().title).toBe("Remove {version}?");
    });

    it("changes every word of every text under the pseudo pack", () => {
        const pack = JSON.parse(readFileSync(PSEUDO, "utf8")) as LanguagePack;
        I18n.adopt(pack);
        const unchanged: string[] = [];
        for (const [key, build] of Object.entries(BUILT)) {
            const english = leaves((en as Record<string, unknown>)[key]);
            const pseudo = new Map(leaves(build()));
            for (const [path, text] of english) {
                if (isWords(path, text) && pseudo.get(path) === text) unchanged.push(`${key}.${path}: ${text}`);
            }
        }
        expect(unchanged).toEqual([]);
    });

    it("words activity as running, last played or never played", () => {
        const base = {
            name: "play",
            folder: "f",
            game: "1.26.60.05",
            preview: false,
            ammlVersion: null,
            ammlLocal: false,
            ammlInstalled: true,
            modCount: 0,
            dataMode: "shared" as const,
            dataDir: null,
            managed: true,
            runningSince: null,
            lastPlayed: null,
            updateCount: 0,
        };
        const now = new Date(2026, 10, 2, 15, 0, 0).getTime();
        const clock = new Date(2026, 10, 2, 14, 2, 0).getTime();

        expect(ActivityTextSet.line({ ...base, runningSince: clock }, now)).toBe("running since 14:02");
        expect(ActivityTextSet.line({ ...base, lastPlayed: now - 2 * 3_600_000 }, now)).toBe("last played 2 h ago");
        expect(ActivityTextSet.line({ ...base, lastPlayed: now - 3 * 86_400_000 }, now)).toBe("last played 3 d ago");
        expect(ActivityTextSet.line(base, now)).toBe("never played");
    });

    it("words a span as minutes, hours, days or months", () => {
        expect(ActivityTextSet.ago(10_000)).toBe("1 min ago");
        expect(ActivityTextSet.ago(4 * 60_000)).toBe("4 min ago");
        expect(ActivityTextSet.ago(5 * 30 * 86_400_000)).toBe("5 mo ago");
        expect(ActivityTextSet.ago(-5)).toBe("1 min ago");
    });
});
