import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the downloads settings page, with every string through I18n. Without a pack it equals the kit English.
export class DownloadsSettingsTextSet {
    static build(): LauncherText["downloadsSettings"] {
        return {
            usage: {
                heading: I18n.t("Downloads folder"),
            },
            filesHeading: I18n.t("Files · {count}"),
            removeUnfinished: I18n.t("Remove unfinished"),
            file: {
                kept: I18n.t("Kept"),
                remove: I18n.t("Remove"),
            },
        };
    }
}
