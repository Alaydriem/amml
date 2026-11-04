import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the new instance screen, with every string through I18n. Without a pack it equals the kit English.
export class NewInstanceTextSet {
    static build(): LauncherText["newInstance"] {
        return {
            back: I18n.t("Instances"),
            title: I18n.t("Create a new **instance**"),
            versionLabel: I18n.t("Game version"),
            release: I18n.t("Release"),
            preview: I18n.t("Preview"),
            versionsLabel: I18n.t("Game versions"),
            versionRow: {
                installedNote: I18n.t("Already in the version store · local copy only"),
                installed: I18n.t("Installed"),
                download: I18n.t("~{size} download"),
                unsupported: I18n.t("Not supported"),
            },
            form: {
                nameLabel: I18n.t("Instance name"),
                folderHint: I18n.t("Folder: {folder}"),
                worldsLabel: I18n.t("Worlds and packs"),
                shared: I18n.t("Shared"),
                own: I18n.t("Own folder"),
                cancel: I18n.t("Cancel"),
                create: I18n.t("Create"),
            },
            folderPath: I18n.t("instances\\{name}"),
            installs: I18n.t("Installs **{amml}** and **{mods}**."),
            amml: I18n.t("AMML {version}"),
            mods: {
                one: I18n.tn("{count} default mod", "{count} default mods", 1),
                other: I18n.tn("{count} default mod", "{count} default mods", 2),
            },
            disk: I18n.t("About {size} on disk."),
        };
    }
}
