import type { InstallProgress, InstallStep, LauncherFault, LauncherFaultText } from "../model";
import type { InstallStepRowText } from "../components/install/InstallStepRow";
import type { ProgressReadoutText } from "../components/install/ProgressReadout";

export interface InstallProgressScreenText {
    /** Back link label ("Instances"). The link shows only when `onback` is set too. */
    back?: string;
    /** Eyebrow above the title. Left out when not set. */
    eyebrow?: string;
    /** Title with the instance name in bold, for example "Creating **{name}**". */
    title: string;
    /** Title of the failed state, for example "Could not create **{name}**". */
    failedTitle: string;
    /** Accessible name of the step list. */
    stepsLabel: string;
    steps: InstallStepRowText;
    /** Template with `{current}` and `{total}` slots, for example "Step {current} of {total}". */
    step: string;
    /** Eyebrow of the failed state, with the same slots, for example "Stopped at step {current} of {total}". */
    stopped: string;
    readout: ProgressReadoutText;
    /** The warning while the install runs. Mark the lead with `**…**`. */
    warning: string;
    /** The note that the Store version is back after a failure. Mark the lead with `**…**`. */
    restored: string;
    /** Cancel label. The button shows only when `oncancel` is set too. */
    cancel?: string;
    retry: string;
    /** Label of the secondary action of the failed state ("Choose another source"). Shows only when `onother` is set too. */
    other?: string;
    showLog: string;
}

export interface InstallProgressScreenProps {
    /** Name of the instance being created. */
    name: string;
    steps: InstallStep[];
    /** Byte progress of the active step. Not drawn in the failed state. */
    progress?: InstallProgress;
    /** Number of the active step, or the step that failed, from 1. */
    stepNumber: number;
    stepTotal: number;
    text: InstallProgressScreenText;
    /** When set, the screen shows the failed state: the fault, the restored note and the retry action. */
    fault?: LauncherFault;
    faultText?: LauncherFaultText;
    /** Shows the note that the Store version was put back. Only drawn in the failed state. */
    restored?: boolean;
    onback?: () => void;
    oncancel?: () => void;
    onretry?: () => void;
    onother?: () => void;
    onshowlog?: () => void;
}
