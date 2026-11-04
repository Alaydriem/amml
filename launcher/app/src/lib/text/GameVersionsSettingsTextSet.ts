import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the game versions settings page, with every string through I18n. Without a pack it equals the kit English.
export class GameVersionsSettingsTextSet {
    static build(): LauncherText["gameVersionsSettings"] {
        return {
            buildsHeading: I18n.t("Builds"),
            previewTitle: I18n.t("Show preview builds"),
            previewNote: I18n.t("Needs Minecraft Preview from the Store. Few mods support them."),
            previewToggleLabel: I18n.t("Show preview builds"),
            listTitle: I18n.t("Version list"),
            listDetail: I18n.t("{source} · {release} release, {preview} preview"),
            refresh: I18n.t("Refresh"),
            installedHeading: I18n.t("Installed versions · {size}"),
            row: {
                instances: {
                    one: I18n.tn("{count} instance", "{count} instances", 1),
                    other: I18n.tn("{count} instance", "{count} instances", 2),
                },
                noInstances: I18n.t("No instances"),
                remove: I18n.t("Remove"),
            },
            note: I18n.t("An installed version is the clean copy every instance of it is made from. A version in use by an instance cannot be removed. Removing one never touches your Store install."),
        };
    }
}
