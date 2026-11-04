export interface DownloadUsageText {
    heading: string;
}

export interface DownloadUsageProps {
    text: DownloadUsageText;
    /** Folder path in the header, mono. */
    path: string;
    /** The big figure, the number alone ("3.34"). */
    value: string;
    /** The unit after the figure, in the faint small style ("GB"). */
    unit: string;
}
