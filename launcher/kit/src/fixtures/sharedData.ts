import type { InstallProgress, InstallStep, LauncherFault } from "../model";

/** Content of artboards 02b and 02c: setting up the shared data on first run. */
export class SharedDataFixtures {
    static readonly name = "Store Minecraft";

    static readonly steps: InstallStep[] = [
        { id: "setup-check", label: "Check the shared data folder", state: "done", note: "0.1 s" },
        { id: "setup-measure", label: "Measure the worlds", state: "done", note: "1.2 s" },
        { id: "setup-copy", label: "Copy into shared data", state: "active", note: "62%" },
        { id: "setup-settings", label: "Save launcher settings", state: "waiting" },
    ];

    /** The same steps when the copy failed (artboard 02c). */
    static readonly failedSteps: InstallStep[] = SharedDataFixtures.steps.map((step) =>
        step.id === "setup-copy" ? { ...step, state: "failed", note: "failed" } : step,
    );

    static readonly progress: InstallProgress = {
        done: 412_000_000,
        total: 664_000_000,
        rate: 38_000_000,
        remaining: 7,
        detail: "%APPDATA%\\Minecraft Bedrock → %LOCALAPPDATA%\\amml\\data\\Minecraft Bedrock",
    };

    static readonly stepNumber = 3;
    static readonly stepTotal = 4;

    static readonly fault: LauncherFault = {
        kind: "download-failed",
        severity: "fault",
        primaryAction: true,
    };
}
