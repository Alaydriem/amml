import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the instance detail screen, with every string through I18n. Without a pack it equals the kit English.
export class InstanceDetailTextSet {
    static build(): LauncherText["instanceDetail"] {
        return {
            sectionLabel: I18n.t("Core · {core} · Mods · {mods}"),
            updateAll: I18n.t("Update all ({count})"),
            addMod: I18n.t("Add a mod"),
            columns: {
                on: I18n.t("On"),
                name: I18n.t("Name"),
                installed: I18n.t("Installed"),
                available: I18n.t("Available"),
                state: I18n.t("Status"),
                actions: I18n.t("Actions"),
            },
            tableLabel: I18n.t("Mods in this instance"),
            row: {
                coreTag: I18n.t("Core"),
                upToDate: I18n.t("Up to date"),
                update: I18n.t("Update"),
                unofficial: I18n.t("Unofficial"),
                localBuild: I18n.t("Local build"),
                noVersion: I18n.tc("separator", "—"),
                toggleLabel: I18n.t("Turn {title} on or off"),
                menuLabel: I18n.t("Actions for {title}"),
                subline: I18n.t("{id} · {note}"),
            },
            menu: [
                {
                    id: "folder",
                    label: I18n.t("Show in folder"),
                },
                {
                    id: "remove",
                    label: I18n.t("Remove"),
                    tone: "fault",
                },
            ],
        };
    }
}
