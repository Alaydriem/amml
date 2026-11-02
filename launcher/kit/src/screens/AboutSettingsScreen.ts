import type { AboutHeaderText, AuthorCardText, BugReportPanelText } from "../components/settings";
import type { AboutAuthor, AboutLink, AboutLinkId, BugReportLine } from "../model";
import { Template } from "../text";

export interface AboutSettingsScreenText {
    header: AboutHeaderText;
    /** `{version}`, `{builtOn}` and `{platform}`. */
    build: string;
    author: AuthorCardText;
    projectHeading: string;
    /** The label and the link text of each project link. */
    links: Record<AboutLinkId, { label: string; value: string }>;
    bugReport: BugReportPanelText;
    disclaimer: string;
}

export interface AboutSettingsScreenProps {
    version: string;
    /** Build date as shown ("2026-11-02"). */
    builtOn: string;
    platform: string;
    /** Newest version when it is newer than `version`; unset when the launcher is up to date. */
    updateVersion?: string;
    author: AboutAuthor;
    links: AboutLink[];
    /** The environment lines of the bug report card. A line with `href` is a link. */
    report: BugReportLine[];
    text: AboutSettingsScreenText;
    onupdate?: () => void;
    /** Called with the address of the author, a project link or the logs folder. */
    onopenlink?: (href: string) => void;
    /** Called with the report text, one line for each entry. */
    oncopyreport?: (report: string) => void;
}

export class AboutSettingsScreenRules {
    static build(props: Pick<AboutSettingsScreenProps, "version" | "builtOn" | "platform">, text: AboutSettingsScreenText): string {
        return Template.fill(text.build, { version: props.version, builtOn: props.builtOn, platform: props.platform });
    }
}
