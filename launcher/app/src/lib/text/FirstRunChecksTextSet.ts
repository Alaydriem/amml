import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the first run system checks screen, with every string through I18n. Without a pack it equals the kit English.
export class FirstRunChecksTextSet {
    static build(): LauncherText["firstRunChecks"] {
        return {
            eyebrow: I18n.t("Get started - AMML"),
            title: I18n.t("Welcome to AMML!"),
            intro: I18n.t("Alaydriem's Modded Minecraft Loader (AMML) is a modded Minecraft loader for Minecraft Bedrock edition that provides a modded version switcher and a mod manager."),
            checksHeading: I18n.t("This PC"),
            check: {
                ok: I18n.t("Passed"),
                warn: I18n.t("Needs attention"),
                muted: I18n.t("Optional"),
            },
            footerNote: I18n.t("The install button may ask for admin permissions to apply and install AMML"),
            skip: I18n.t("Skip"),
            next: I18n.t("Next"),
        };
    }
}
