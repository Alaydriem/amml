/** Byte progress of the active install step. */
export interface InstallProgress {
    done: number;
    total: number;
    /** Bytes per second; undefined while unknown. */
    rate?: number;
    /** Seconds left; undefined while unknown. */
    remaining?: number;
    /** Line under the bar, for example `mirror 1 of 2 · launcher\downloads\1.26.51.1.msixvc`. */
    detail?: string;
}
