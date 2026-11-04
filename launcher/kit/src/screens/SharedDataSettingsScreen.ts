import type { SharedFolderPanelText, SharedFolderStat, WorldLocationRowText, WorldSourceRowText } from "../components/settings";
import type { SharedDataStats, WorldLocation, WorldSource } from "../model";
import { Template, Units, type Plural } from "../text";

export interface SharedDataSettingsScreenText {
    folder: SharedFolderPanelText;
    locationsHeading: string;
    location: WorldLocationRowText;
    copyInHeading: string;
    source: WorldSourceRowText;
    /** `{count}` worlds in a source. */
    worlds: Plural;
    /** `{folder}`, `{worlds}` and `{modified}`. */
    sourceDetail: string;
    note: string;
}

export interface SharedDataSettingsScreenProps {
    stats: SharedDataStats;
    locations: WorldLocation[];
    /** Folders the user can copy worlds from. */
    sources: WorldSource[];
    text: SharedDataSettingsScreenText;
    onopenfolder?: () => void;
    oncopyworlds?: (id: string) => void;
}

export class SharedDataSettingsScreenRules {
    /** The four totals of the shared folder card. */
    static stats(stats: SharedDataStats): SharedFolderStat[] {
        const disk = Units.en.sizeParts(stats.bytes, { trim: true });
        return [
            { id: "worlds", value: Units.en.count(stats.worlds) },
            { id: "resourcePacks", value: Units.en.count(stats.resourcePacks) },
            { id: "behaviorPacks", value: Units.en.count(stats.behaviorPacks) },
            { id: "onDisk", value: disk.value, unit: disk.unit },
        ];
    }

    static sourceDetail(source: WorldSource, text: SharedDataSettingsScreenText): string {
        return Template.fill(text.sourceDetail, {
            folder: source.folder,
            worlds: Template.plural(text.worlds, source.worldCount),
            modified: source.modified,
        });
    }
}
