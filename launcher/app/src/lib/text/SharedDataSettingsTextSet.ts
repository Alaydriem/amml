import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the shared data settings page, with every string through I18n. Without a pack it equals the kit English.
export class SharedDataSettingsTextSet {
    static build(): LauncherText["sharedDataSettings"] {
        return {
            folder: {
                heading: I18n.t("Shared folder"),
                note: I18n.t("Every instance set to Shared reads and writes here."),
                open: I18n.t("Open"),
                stats: {
                    worlds: I18n.t("Worlds"),
                    resourcePacks: I18n.t("Resource packs"),
                    behaviorPacks: I18n.t("Behavior packs"),
                    onDisk: I18n.t("On disk"),
                },
            },
            locationsHeading: I18n.t("Where each instance keeps its worlds"),
            location: {
                shared: I18n.t("Shared"),
                own: I18n.t("Own folder"),
            },
            copyInHeading: I18n.t("Copy worlds in"),
            source: {
                copy: I18n.t("Copy"),
            },
            worlds: {
                one: I18n.tn("{count} world", "{count} worlds", 1),
                other: I18n.tn("{count} world", "{count} worlds", 2),
            },
            sourceDetail: I18n.t("{folder} · {worlds} · {modified}"),
            note: I18n.t("Copying never moves or deletes the source, and never overwrites a world already in the shared folder."),
        };
    }
}
