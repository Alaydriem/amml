/** One step of an install (artboard 05). */
export interface InstallStep {
    id: string;
    label: string;
    state: "done" | "active" | "waiting" | "failed";
    /** Right-hand note, for example "0.4 s", "cached" or "62%". */
    note?: string;
}
