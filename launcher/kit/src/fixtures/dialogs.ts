import type { AddModSuggestion, LauncherFault, LauncherFaultKind } from "../model";

/** Data of artboard 08. */
export class DialogFixtures {
    static readonly developerModeFile = "Minecraft.Windows.exe";
    static readonly developerModeVersion = "1.26.51.1";
    static readonly deleteName = "1.26.52.3-prores";
    static readonly deleteFolder = "%LOCALAPPDATA%\\amml\\instances\\1.26.52.3-prores";
    static readonly addModInstance = "1.26.60.05";
    static readonly addModInput = "https://example.org/nightvision-1.2.0.zip";

    static readonly knownIds: string[] = ["@alaydriem/peek", "@alaydriem/lightlevels", "@alaydriem/zoom"];

    static readonly suggestions: AddModSuggestion[] = [
        { id: "@alaydriem/peek", title: "Peek" },
        { id: "@alaydriem/lightlevels", title: "Light Levels" },
    ];

    static readonly faults: Record<LauncherFaultKind, LauncherFault> = {
        "store-missing": { kind: "store-missing", severity: "fault", primaryAction: false },
        "gaming-services-missing": { kind: "gaming-services-missing", severity: "fault", primaryAction: false },
        "preview-missing": { kind: "preview-missing", severity: "warn", primaryAction: false },
        "download-failed": { kind: "download-failed", severity: "fault", primaryAction: true },
        "deploy-refused": { kind: "deploy-refused", severity: "fault", code: "0x80073CF3 · package failed updates", primaryAction: false },
        "copy-failed": { kind: "copy-failed", severity: "fault", primaryAction: true },
        "disk-space": { kind: "disk-space", severity: "warn", primaryAction: false },
        "install-interrupted": { kind: "install-interrupted", severity: "info", primaryAction: false },
        "restore-failed": {
            kind: "restore-failed",
            severity: "fault",
            code: "C:\\amml\\launcher\\downloads\\Microsoft.MinecraftUWP_1.26.60.5.msixvc",
            primaryAction: false,
        },
        "operation-failed": { kind: "operation-failed", severity: "fault", code: "file error: access is denied", primaryAction: false },
    };

    /** The banners in the order of the artboard. */
    static readonly faultOrder: LauncherFaultKind[] = [
        "store-missing",
        "gaming-services-missing",
        "preview-missing",
        "download-failed",
        "deploy-refused",
        "copy-failed",
        "disk-space",
        "install-interrupted",
        "restore-failed",
        "operation-failed",
    ];
}
