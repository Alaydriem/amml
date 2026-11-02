import type { DownloadFileRowText, DownloadUsageText } from "../components/settings";
import type { DownloadFile, Option } from "../model";
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
    usedBytes: number;
    /** The limit in bytes; the bar shows `usedBytes` against it. */
    limitBytes: number;
    /** Id of the chosen limit option. The select shows it and never changes it on its own. */
    limit: string;
    limitOptions: Option[];
    files: DownloadFile[];
    text: DownloadsSettingsScreenText;
    onlimitchange?: (limit: string) => void;
    onremovefile?: (id: string) => void;
    onremoveunfinished?: () => void;
}

export class DownloadsSettingsScreenRules {
    /** The big figure: the number alone when its unit is the limit's unit (`3.34`), else number and unit. */
    static usedText(usedBytes: number, limitBytes: number): string {
        const used = Units.en.sizeParts(usedBytes, { trim: true });
        const limit = Units.en.sizeParts(limitBytes, { trim: true });
        return used.unit === limit.unit ? used.value : Units.en.size(usedBytes, { trim: true });
    }

    static percent(usedBytes: number, limitBytes: number): number {
        return Units.percent(usedBytes, limitBytes);
    }

    /** A file size in GB with two decimals, whatever its size (`0.02 GB`). */
    static fileSize(bytes: number): string {
        return Units.en.size(bytes, { unit: "gb" });
    }

    static hasUnfinished(files: DownloadFile[]): boolean {
        return files.some((file) => file.state === "unfinished");
    }
}
