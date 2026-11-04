import type { UrlCheck } from "../../model";
import { Template, type TextPart } from "../../text/Template";
import type { CheckRowText } from "../setup/CheckRow";

export interface UrlInstallPaneText {
    /** The sentence above the field. */
    intro: string;
    urlLabel: string;
    check: string;
    /** The mono caps header of the check panel. */
    checksLabel: string;
    /** What a screen reader says for each check mark. */
    checkMarks: CheckRowText;
    /** The warn callout under the checks; `**…**` marks the strong part. */
    unofficial: string;
}

export interface UrlInstallPaneProps {
    /** Shown link. The field never changes it: the parent answers `oninput` with a new value. */
    url?: string;
    checks: UrlCheck[];
    text: UrlInstallPaneText;
    /** Shows the warn callout under the checks. */
    unofficial?: boolean;
    /** The Check button is disabled while a check runs or the field is empty. */
    checking?: boolean;
    oninput?: (url: string) => void;
    oncheck?: (url: string) => void;
}

export class UrlInstallPaneRules {
    static canCheck(url: string, checking: boolean): boolean {
        return !checking && url.trim().length > 0;
    }

    static unofficial(text: UrlInstallPaneText): TextPart[] {
        return Template.parts(text.unofficial);
    }
}
