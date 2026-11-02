import type { InstallStep } from "../../model";
import type { StatusMarkState } from "../primitives/StatusMark";

export interface InstallStepRowText {
    /** Read by assistive technology for each step state ("Done", "In progress", "Waiting", "Failed"). */
    done: string;
    active: string;
    waiting: string;
    failed: string;
}

export interface InstallStepRowProps {
    step: InstallStep;
    text: InstallStepRowText;
}

export class InstallStepRowRules {
    /** The mark of a step. */
    static mark(state: InstallStep["state"]): StatusMarkState {
        switch (state) {
            case "done":
                return "ok";
            case "active":
                return "active";
            case "waiting":
                return "pending";
            case "failed":
                return "failed";
        }
    }
}
