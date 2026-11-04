import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the title bar, status bar, settings nav and context chip, with every string through I18n. Without a pack it equals the kit English.
export class ShellTextSet {
    static build(): LauncherText["shell"] {
        return {
            titleBar: {
                wordmark: I18n.t("AMML Launcher"),
                refresh: I18n.t("Refresh registries"),
                settings: I18n.t("Settings"),
                step: I18n.t("Setup · {current} of {total}"),
                registry: I18n.t("Registry · {fetched}"),
            },
            statusBar: {
                left: I18n.t("launcher {version} · data {dataPath}"),
                store: I18n.t("Store Bedrock {version} registered"),
            },
            settingsNav: {
                back: I18n.t("Instances"),
                label: I18n.t("Settings sections"),
            },
            settingsSections: [
                {
                    id: "registries",
                    label: I18n.t("Registries"),
                },
                {
                    id: "versions",
                    label: I18n.t("Game versions"),
                },
                {
                    id: "shared",
                    label: I18n.t("Shared data"),
                },
                {
                    id: "downloads",
                    label: I18n.t("Downloads"),
                },
                {
                    id: "about",
                    label: I18n.t("About"),
                },
            ],
            contextChip: {
                separator: I18n.tc("separator", "·"),
            },
        };
    }
}
