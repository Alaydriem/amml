import type { LauncherFault, LauncherFaultKind, LauncherFaultText } from "../../model";

export interface FaultBannerProps {
    fault: LauncherFault;
    text: LauncherFaultText;
    /** Called with the fault's kind when the user presses the action. */
    onaction?: (kind: LauncherFaultKind) => void;
}
