import type { AddModUrlScreenText } from "../../screens/AddModUrlScreen";
import { addModBrowseEn } from "./addModBrowse";
import { firstRunChecksEn } from "./firstRunChecks";

export const addModUrlEn: AddModUrlScreenText = {
    header: addModBrowseEn.header,
    nav: addModBrowseEn.nav,
    pane: {
        intro: "Paste a link to a mod's release zip. If the link is in a registry, it installs as that registry's mod.",
        urlLabel: "Link to the release zip",
        check: "Check",
        checksLabel: "What the launcher checked",
        checkMarks: firstRunChecksEn.check,
        unofficial:
            "**This installs as unofficial.** Nobody has checked this file, and the launcher never looks for updates to it. Only install mods from people you trust.",
    },
    found: "**{label}** · {id}",
    installUnofficial: "Install unofficial",
};
