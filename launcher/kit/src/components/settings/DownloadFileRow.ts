export interface DownloadFileRowText {
    /** Tag of a file the launcher keeps; it cannot be removed. */
    kept: string;
    remove: string;
}

export interface DownloadFileRowProps {
    id: string;
    /** File name, mono. */
    name: string;
    /** Second line ("Unfinished · 62% · resumes on the next install"). */
    detail: string;
    /** Size as shown, already formatted. */
    sizeText: string;
    /** A kept file shows the `kept` tag and cannot be removed. */
    kept?: boolean;
    text: DownloadFileRowText;
    onremove?: (id: string) => void;
}
