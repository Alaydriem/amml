import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the first run adopt screen, with every string through I18n. Without a pack it equals the kit English.
export class FirstRunAdoptTextSet {
    static build(): LauncherText["firstRunAdopt"] {
        return {
            eyebrow: I18n.t("Adopt current data"),
            title: I18n.t("Keep your instances. **Share your worlds.**"),
            intro: I18n.t("AMML can adopt data from your current Minecraft Bedrock install, or for another install found on disk, or create a clean instance without any data"),
            foundHeading: I18n.t("Instances found · adopted as they are"),
            copyHeading: I18n.t("Copy into shared data"),
            adopted: {
                meta: I18n.t("AMML {amml} · {mods}"),
                unknownVersion: I18n.t("unknown version"),
                mods: {
                    one: I18n.tn("{count} mod", "{count} mods", 1),
                    other: I18n.tn("{count} mod", "{count} mods", 2),
                },
            },
            footerNote: I18n.t("Nothing is moved or deleted. You can change this later in Settings."),
            back: I18n.t("Back"),
            next: I18n.t("Next"),
        };
    }
}
