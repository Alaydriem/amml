/** The words of one fault. `en.faults` maps each `LauncherFaultKind` to one of these. */
export interface LauncherFaultText {
    /** Bold lead sentence. */
    title: string;
    body: string;
    /** Label of the action button. */
    action: string;
}
