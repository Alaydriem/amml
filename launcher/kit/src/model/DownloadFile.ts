/** One file in the downloads folder (artboard 07d). */
export interface DownloadFile {
    id: string;
    /** File name, as shown. */
    name: string;
    size: number;
    state: "kept" | "unfinished" | "waiting";
    detail: string;
}
