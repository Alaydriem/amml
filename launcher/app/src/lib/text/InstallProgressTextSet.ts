import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the install progress screen, with every string through I18n. Without a pack it equals the kit English.
export class InstallProgressTextSet {
    static build(): LauncherText["installProgress"] {
        return {
            back: I18n.t("Instances"),
            title: I18n.t("Creating **{name}**"),
            failedTitle: I18n.t("Could not create **{name}**"),
            stepsLabel: I18n.t("Install steps"),
            steps: {
                done: I18n.t("Done"),
                active: I18n.t("In progress"),
                waiting: I18n.t("Waiting"),
                failed: I18n.t("Failed"),
            },
            step: I18n.t("Step {current} of {total}"),
            stopped: I18n.t("Stopped at step {current} of {total}"),
            readout: {
                progressLabel: I18n.t("Download progress"),
                timeLeft: I18n.t("about {time} left"),
                separator: I18n.tc("separator", " · "),
            },
            warning: I18n.t("**AMML is creating your new instance. Do not start Minecraft from the Store during this install.** This may take a minute or two to complete."),
            restored: I18n.t("**Your Store version was put back.** It starts as before. No instance folder was made."),
            cancel: I18n.t("Cancel install"),
            retry: I18n.t("Try again"),
            showLog: I18n.t("Show log"),
        };
    }
}
