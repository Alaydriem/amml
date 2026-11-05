import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the instance list, with every string through I18n. Without a pack it equals the kit English.
export class InstancesTextSet {
    static build(): LauncherText["instances"] {
        return {
            eyebrow: I18n.t("Instances · {count}"),
            title: I18n.t("Launch **AMML.**"),
            newInstance: I18n.t("New instance"),
            banner: {
                message: I18n.t("**{updates} available** in {instances}. Nothing is installed until you ask."),
                updates: {
                    one: I18n.tn("{count} update", "{count} updates", 1),
                    other: I18n.tn("{count} update", "{count} updates", 2),
                },
                instances: {
                    one: I18n.tn("{count} instance", "{count} instances", 1),
                    other: I18n.tn("{count} instance", "{count} instances", 2),
                },
                review: I18n.t("Review"),
            },
            listLabel: I18n.t("Instances"),
            empty: {
                title: I18n.t("No instances yet."),
                body: I18n.t("An instance is one game version with AMML and its mods, in a folder of its own. Make as many as you like. Your Store install is never changed."),
            },
            row: InstancesTextSet.instanceRow(),
        };
    }

    static instanceRow(): LauncherText["instanceRow"] {
        return {
            game: I18n.t("Game"),
            amml: I18n.t("AMML"),
            mods: I18n.t("Mods"),
            launch: I18n.t("Launch"),
            running: I18n.t("Running"),
            upToDate: I18n.t("Up to date"),
            localBuild: I18n.t("Local build"),
            updates: {
                one: I18n.tn("{count} update", "{count} updates", 1),
                other: I18n.tn("{count} update", "{count} updates", 2),
            },
            activityLine: I18n.t("{data} · {activity}"),
            dataShared: I18n.t("Shared data"),
            dataOwn: I18n.t("Own data folder"),
            menuLabel: I18n.t("More actions"),
        };
    }

    static instanceMenu(): LauncherText["instanceMenu"] {
        return [
            {
                id: "open-folder",
                label: I18n.t("Open folder"),
            },
            {
                id: "delete",
                label: I18n.t("Delete"),
                tone: "fault",
            },
        ];
    }

    static instanceHeader(): LauncherText["instanceHeader"] {
        return {
            back: I18n.t("Instances"),
            delete: I18n.t("Delete"),
            openFolder: I18n.t("Open folder"),
            launch: I18n.t("Launch"),
            running: I18n.t("Running"),
            detail: I18n.t("Game {game} · {data} · {folder}"),
            dataShared: I18n.t("shared data"),
            dataOwn: I18n.t("own data folder"),
        };
    }
}
