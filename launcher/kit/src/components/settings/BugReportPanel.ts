import type { BugReportLine, BugReportLineId } from "../../model";
import { Template } from "../../text/Template";

export interface BugReportPanelText {
    heading: string;
    copy: string;
    /** One line of the copied report; `{label}` and `{value}`. */
    line: string;
    /** Shown value of a line that is a link; `{value}`. */
    linkValue: string;
    /** The label of each line. */
    labels: Record<BugReportLineId, string>;
}

export interface BugReportPanelProps {
    text: BugReportPanelText;
    lines: BugReportLine[];
    /** Called with the report text, one line for each entry, in order. */
    oncopy?: (report: string) => void;
    /** Called with `href` when the user presses a line that has one (the logs folder). */
    onopenlink?: (href: string) => void;
}

export class BugReportPanelRules {
    /** The copied report: one `line` template for each entry, in order, with the plain value. */
    static report(lines: BugReportLine[], text: BugReportPanelText): string {
        return lines
            .map((line) => Template.fill(text.line, { label: text.labels[line.id], value: line.value }))
            .join("\n");
    }

    /** The value as drawn: a link gets the `linkValue` template. */
    static shown(line: BugReportLine, text: BugReportPanelText): string {
        return line.href === undefined ? line.value : Template.fill(text.linkValue, { value: line.value });
    }
}
