import type { InstallProgress, InstallStep, LauncherFault, LauncherFaultText } from "../model";
import type { InstallStepRowText } from "../components/install/InstallStepRow";
import type { ProgressReadoutText } from "../components/install/ProgressReadout";

export interface InstallProgressScreenText {
    /** Back link label ("Instances"). */
    back: string;
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
    cancel: string;
    retry: string;
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
    onshowlog?: () => void;
}
