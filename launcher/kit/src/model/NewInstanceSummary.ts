/** What the new instance will contain, as the app works it out for the chosen version. */
export interface NewInstanceSummary {
    ammlVersion: string;
    defaultMods: number;
    /** Size on disk in bytes. */
    diskBytes: number;
}
