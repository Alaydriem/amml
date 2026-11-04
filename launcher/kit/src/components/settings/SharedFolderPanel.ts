export type SharedFolderStatId = "worlds" | "resourcePacks" | "behaviorPacks" | "onDisk";

/** One total under the folder path. Its caption is `text.stats[id]`. */
export interface SharedFolderStat {
    id: SharedFolderStatId;
    /** The figure, already formatted ("14", "3.2"). */
    value: string;
    /** Unit after the figure ("GB"). */
    unit?: string;
}

export interface SharedFolderPanelText {
    heading: string;
    note: string;
    open: string;
    /** The caption of each total. */
    stats: Record<SharedFolderStatId, string>;
}

export interface SharedFolderPanelProps {
    text: SharedFolderPanelText;
    /** Folder path, mono. */
    path: string;
    stats: SharedFolderStat[];
    onopen?: () => void;
}
