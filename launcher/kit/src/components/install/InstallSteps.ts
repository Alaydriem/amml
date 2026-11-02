import type { InstallStep } from "../../model";
import type { InstallStepRowText } from "./InstallStepRow";

export interface InstallStepsProps {
    steps: InstallStep[];
    text: InstallStepRowText;
    /** Accessible name of the list. */
    label: string;
}
