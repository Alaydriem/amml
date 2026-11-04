import type { DownloadFileRowText, DownloadUsageText } from "../components/settings";
import type { DownloadFile } from "../model";
import { Units } from "../text";

export interface DownloadsSettingsScreenText {
    usage: DownloadUsageText;
    /** `{count}` files in the folder. */
    filesHeading: string;
    removeUnfinished: string;
    file: DownloadFileRowText;
}

export interface DownloadsSettingsScreenProps {
    /** Downloads folder, as shown. */
    folder: string;
    /** Total size of the folder in bytes, shown as one figure. */
    usedBytes: number;
    files: DownloadFile[];
    text: DownloadsSettingsScreenText;
    onremovefile?: (id: string) => void;
    onremoveunfinished?: () => void;
}

export class DownloadsSettingsScreenRules {
    /** The total as a big number and a small unit: `{ value: "3.34", unit: "GB" }`. */
    static used(usedBytes: number): { value: string; unit: string } {
        return Units.en.sizeParts(usedBytes, { trim: true });
    }

    /** A file size in GB with two decimals, whatever its size (`0.02 GB`). */
    static fileSize(bytes: number): string {
        return Units.en.size(bytes, { unit: "gb" });
    }

    static hasUnfinished(files: DownloadFile[]): boolean {
        return files.some((file) => file.state === "unfinished");
    }
}
