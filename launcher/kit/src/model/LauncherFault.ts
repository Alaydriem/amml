import type { Severity } from "./Severity";

/** The named error states of artboard 08. The app maps each Rust error to one. */
export type LauncherFaultKind =
    | "store-missing"
    | "gaming-services-missing"
    | "preview-missing"
    | "download-failed"
    | "deploy-refused"
    | "copy-failed"
    | "disk-space"
    | "install-interrupted";

export interface LauncherFault {
    kind: LauncherFaultKind;
    severity: Extract<Severity, "fault" | "warn" | "info">;
    /** Mono detail after the body, for example `0x80073CF3 · package failed updates`. */
    code?: string;
    /** The action is the primary next step (filled button). */
    primaryAction: boolean;
}
