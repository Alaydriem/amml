import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the about settings page, with every string through I18n. Without a pack it equals the kit English.
export class AboutSettingsTextSet {
    static build(): LauncherText["aboutSettings"] {
        return {
            header: {
                title: I18n.t("AMML **Launcher**"),
                markLabel: I18n.t("AMML Launcher"),
                updateNotice: I18n.t("{version} is out"),
                update: I18n.t("Update and restart"),
            },
            build: I18n.t("{version} · built {builtOn} · {platform}"),
            author: {
                byline: I18n.t("Made by {name}"),
                open: I18n.t("GitHub"),
            },
            projectHeading: I18n.t("Project"),
            links: {
                source: {
                    label: I18n.t("Source code"),
                    value: I18n.t("Alaydriem/amml"),
                },
                issues: {
                    label: I18n.t("Report a problem"),
                    value: I18n.t("GitHub issues"),
                },
                licence: {
                    label: I18n.t("Licence"),
                    value: I18n.t("BSD 3-Clause"),
                },
                notices: {
                    label: I18n.t("Third-party notices"),
                    value: I18n.t("Tauri, Svelte, fonts"),
                },
            },
            bugReport: {
                heading: I18n.t("For a bug report"),
                copy: I18n.t("Copy all"),
                line: I18n.t("{label}: {value}"),
                linkValue: I18n.t("{value} · Open"),
                labels: {
                    windows: I18n.t("Windows"),
                    store: I18n.t("Store Bedrock"),
                    devMode: I18n.t("Developer Mode"),
                    registry: I18n.t("Registry"),
                    logs: I18n.t("Logs"),
                },
            },
            disclaimer: I18n.t("Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft. The game is installed from your own Store licence."),
        };
    }
}
