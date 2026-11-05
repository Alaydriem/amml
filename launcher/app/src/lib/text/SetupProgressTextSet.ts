import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";
import { InstallProgressTextSet } from "./InstallProgressTextSet.ts";

// The text of the first-run copy progress screen, with every string through I18n. Without a pack it equals the kit English.
export class SetupProgressTextSet {
    // `restored` is true when a failed copy was undone; false when a stopped merge left files in shared data
    static build(restored: boolean = true): LauncherText["installProgress"] {
        return {
            ...InstallProgressTextSet.build(),
            back: undefined,
            cancel: undefined,
            eyebrow: I18n.t("Setting up shared data"),
            title: I18n.t("Copying worlds from **{name}**"),
            failedTitle: I18n.t("Could not copy worlds from **{name}**"),
            stepsLabel: I18n.t("Setup steps"),
            readout: {
                progressLabel: I18n.t("Copy progress"),
                timeLeft: I18n.t("about {time} left"),
                separator: I18n.tc("separator", " · "),
            },
            warning: I18n.t(
                "**AMML is copying your worlds into shared data. Do not start Minecraft from the Store during this copy.** Nothing is moved or deleted: your Store worlds stay where they are.",
            ),
            restored: restored
                ? I18n.t("**Nothing was changed.** Your Store worlds are where they were. The part copy was removed and no settings were saved.")
                : I18n.t("**The files copied so far stay in shared data.** Try again copies the rest. Your Store worlds are where they were."),
            other: I18n.t("Choose another source"),
        };
    }
}
