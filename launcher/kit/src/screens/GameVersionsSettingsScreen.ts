import type { InstalledVersionRowText } from "../components/settings";
import type { GameVersion } from "../model";
import { Template, Units } from "../text";

export interface GameVersionsSettingsScreenText {
    buildsHeading: string;
    previewTitle: string;
    previewNote: string;
    /** Accessible name of the preview switch. */
    previewToggleLabel: string;
    listTitle: string;
    /** `{source}`, `{release}` and `{preview}`. */
    listDetail: string;
    refresh: string;
    /** `{size}`: the size of every installed version together. */
    installedHeading: string;
    row: InstalledVersionRowText;
    note: string;
}

export interface GameVersionsSettingsScreenProps {
    /** Whether preview builds are listed. The switch shows it and never changes it on its own. */
    showPreview: boolean;
    /** Where the version list comes from, as shown. */
    versionSource: string;
    releaseCount: number;
    previewCount: number;
    /** When the list was fetched ("4 min ago"). */
    fetched: string;
    /** Versions on disk. Entries that are not `installed` are left out. */
    versions: GameVersion[];
    /** Folder the versions live in, as shown. */
    versionsFolder: string;
    text: GameVersionsSettingsScreenText;
    onpreviewchange?: (on: boolean) => void;
    onrefreshversions?: () => void;
    onremoveversion?: (version: string) => void;
}

export class GameVersionsSettingsScreenRules {
    static installed(versions: GameVersion[]): GameVersion[] {
        return versions.filter((version) => version.state === "installed");
    }

    static totalBytes(versions: GameVersion[]): number {
        return GameVersionsSettingsScreenRules.installed(versions).reduce((sum, version) => sum + (version.size ?? 0), 0);
    }

    static installedHeading(versions: GameVersion[], text: GameVersionsSettingsScreenText): string {
        const total = GameVersionsSettingsScreenRules.totalBytes(versions);
        return Template.fill(text.installedHeading, { size: Units.en.size(total, { trim: true }) });
    }

    static listDetail(
        props: Pick<GameVersionsSettingsScreenProps, "versionSource" | "releaseCount" | "previewCount">,
        text: GameVersionsSettingsScreenText,
    ): string {
        return Template.fill(text.listDetail, {
            source: props.versionSource,
            release: props.releaseCount,
            preview: props.previewCount,
        });
    }
}
