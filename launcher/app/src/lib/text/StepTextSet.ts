import type { JobStepView } from "#lib/bindings/JobStepView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { Units, type InstallStep } from "@amml/launcher-kit";
import { UnitsTextSet } from "./UnitsTextSet.ts";

// The words of a step of a job: its name, and the note on its right.
export class StepTextSet {
    static readonly MOD_PREFIX = "mod:";

    // `version` is the game version the job makes, for the steps that name it
    static label(id: string, version: string | null): string {
        const game = version ?? "";
        switch (id) {
            case "version-check":
                return I18n.t("Check the Store version");
            case "version-download-current":
                return I18n.t("Downloading your installed version");
            case "version-download-wanted":
                return I18n.tf("Download {version}", { version: game });
            case "version-install":
                return I18n.tf("Installing {version}", { version: game });
            case "version-copy-exe":
                return I18n.t("Copying the game executable");
            case "version-copy-files":
                return I18n.t("Copying the game files");
            case "version-remove":
                return I18n.tf("Removing {version} from the Store install", { version: game });
            case "version-restore":
                return I18n.t("Putting your Store version back");
            case "version-verify":
                return I18n.t("Check the Store install");
            case "manager-wait":
                return I18n.t("Waiting for another task to end");
            case "instance":
                return I18n.t("Creating the instance");
            case "amml-download":
                return I18n.t("Download AMML");
            case "amml-patch":
                return I18n.t("Installing AMML");
            case "finish":
                return I18n.t("Finishing");
            case "mod-url":
                return I18n.t("Installing the mod");
            case "worlds":
                return I18n.t("Copying worlds");
            case "game-input":
                return I18n.t("Installing GameInput");
            case "setup-check":
                return I18n.t("Check the shared data folder");
            case "setup-measure":
                return I18n.t("Measure the worlds");
            case "setup-copy":
                return I18n.t("Copy into shared data");
            case "setup-settings":
                return I18n.t("Save launcher settings");
            case "setup-data":
                return I18n.t("Setting up your data");
        }
        if (id.startsWith(StepTextSet.MOD_PREFIX)) {
            return I18n.tf("Installing {id}", { id: id.slice(StepTextSet.MOD_PREFIX.length) });
        }
        return id;
    }

    // "0.4 s" for a quick step, "cached", "62%" or "failed"
    static note(step: JobStepView): string | undefined {
        if (step.state === "failed") return I18n.t("failed");
        if (step.state === "active") {
            return step.percent === null ? undefined : I18n.tf("{percent}%", { percent: step.percent });
        }
        if (step.cached) return I18n.t("cached");
        if (step.state === "done" && step.millis !== null) {
            const units = new Units(UnitsTextSet.build());
            if (step.millis < 1000) return StepTextSet.tenths(units, step.millis);
            return units.duration(step.millis / 1000);
        }
        return undefined;
    }

    // a step shorter than a second shows tenths: "0.4 s"
    private static tenths(units: Units, millis: number): string {
        return units.text.seconds.replace("{n}", (millis / 1000).toFixed(1));
    }

    static toInstallSteps(steps: readonly JobStepView[], version: string | null): InstallStep[] {
        return steps.map((step) => ({
            id: step.id,
            label: StepTextSet.label(step.id, version),
            state: step.state,
            note: StepTextSet.note(step),
        }));
    }
}
