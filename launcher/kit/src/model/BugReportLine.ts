/** The lines of the bug report card, in order (artboard 07e). Their labels are in `en.aboutSettings`. */
export type BugReportLineId = "windows" | "store" | "devMode" | "registry" | "logs";

/** One line of the bug report: a fact about the machine, or a link such as the logs folder. */
export interface BugReportLine {
    id: BugReportLineId;
    value: string;
    /** When set the value is a link; the address is passed to `onopenlink`. */
    href?: string;
}
