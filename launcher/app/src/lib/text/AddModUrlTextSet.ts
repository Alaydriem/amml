import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";
import { AddModBrowseTextSet } from "./AddModBrowseTextSet.ts";
import { FirstRunChecksTextSet } from "./FirstRunChecksTextSet.ts";

// The text of the add a mod from a URL screen, with every string through I18n. Without a pack it equals the kit English.
export class AddModUrlTextSet {
    static build(): LauncherText["addModUrl"] {
        return {
            header: AddModBrowseTextSet.build().header,
            nav: AddModBrowseTextSet.build().nav,
            pane: {
                intro: I18n.t("Paste a link to a mod's release zip. If the link is in a registry, it installs as that registry's mod."),
                urlLabel: I18n.t("Link to the release zip"),
                check: I18n.t("Check"),
                checksLabel: I18n.t("What the launcher checked"),
                checkMarks: FirstRunChecksTextSet.build().check,
                unofficial: I18n.t("**This installs as unofficial.** Nobody has checked this file, and the launcher never looks for updates to it. Only install mods from people you trust."),
            },
            found: I18n.t("**{label}** · {id}"),
            installUnofficial: I18n.t("Install unofficial"),
        };
    }
}
