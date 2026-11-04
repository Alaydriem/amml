import type { SystemCheck } from "../model";
import type { CheckRowText } from "../components/setup/CheckRow";

export interface FirstRunChecksScreenText {
    /** Mono line above the title ("Get started - AMML"). */
    eyebrow: string;
    title: string;
    intro: string;
    /** Panel header of the checks ("This PC"). */
    checksHeading: string;
    check: CheckRowText;
    /** Footer note on the left. */
    footerNote: string;
    skip: string;
    next: string;
}

export interface FirstRunChecksScreenProps {
    checks: SystemCheck[];
    /** Holds Next back, for example until the Store game and Gaming Services are found. Skip stays on. */
    nextDisabled?: boolean;
    text: FirstRunChecksScreenText;
    /** The action button of a check was pressed. */
    oninstallcheck?: (id: string) => void;
    onnext?: () => void;
    onskip?: () => void;
}
