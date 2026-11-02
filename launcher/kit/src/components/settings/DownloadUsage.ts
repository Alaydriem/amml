import type { Option } from "../../model";
import { Template } from "../../text/Template";

export interface DownloadUsageText {
    heading: string;
    /** Note on the right of the figure. */
    note: string;
    /** After the figure; `{limit}` is the label of the chosen limit ("of {limit}"). */
    ofLimit: string;
    /** Accessible name of the usage bar. */
    barLabel: string;
    limitTitle: string;
    limitNote: string;
    /** Accessible name of the limit select. */
    limitLabel: string;
}

export interface DownloadUsageProps {
    text: DownloadUsageText;
    /** Folder path in the header, mono. */
    path: string;
    /** The big figure ("3.34"). */
    usedText: string;
    /** Used share of the limit, 0 to 100. */
    percent: number;
    limitOptions: Option[];
    /** Id of the chosen limit option. The select shows it and never changes it on its own. */
    limit: string;
    /** Called with the id the user picked. */
    onlimitchange?: (limit: string) => void;
}

export class DownloadUsageRules {
    /** The faint text after the figure, from the label of the chosen limit. */
    static ofText(limit: string, options: Option[], text: DownloadUsageText): string {
        const label = options.find((option) => option.id === limit)?.label ?? limit;
        return Template.fill(text.ofLimit, { limit: label });
    }
}
