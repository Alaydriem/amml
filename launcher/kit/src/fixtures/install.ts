import type { InstallProgress, InstallStep, LauncherFault } from "../model";

/** Content of artboards 05 and 05b. */
export class InstallFixtures {
    static readonly name = "1.26.51.1-players";

    static readonly steps: InstallStep[] = [
        { id: "check", label: "Check the Store version", state: "done", note: "0.4 s" },
        { id: "current", label: "Downloading your current instance", state: "done", note: "cached" },
        { id: "download", label: "Download 1.26.51.1", state: "active", note: "62%" },
        { id: "game", label: "Installing 1.26.51.1", state: "waiting" },
        { id: "amml", label: "Installing AMML", state: "waiting" },
    ];

    /** The same steps when the download failed (artboard 05b). */
    static readonly failedSteps: InstallStep[] = InstallFixtures.steps.map((step) =>
        step.id === "download" ? { ...step, state: "failed", note: "failed" } : step,
    );

    static readonly progress: InstallProgress = {
        done: 1_280_000_000,
        total: 2_060_000_000,
        rate: 38_000_000,
        remaining: 20,
        detail: "mirror 1 of 2 · launcher\\downloads\\1.26.51.1.msixvc",
    };

    /** The artboard reads "Step 3 of 6" above a list of five rows. */
    static readonly stepNumber = 3;
    static readonly stepTotal = 6;

    /** The fault of artboard 05b. Its text is `en.faults["download-failed"]`. */
    static readonly fault: LauncherFault = {
        kind: "download-failed",
        severity: "fault",
        primaryAction: true,
    };
}
