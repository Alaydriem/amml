import type { InstallProgress } from "../../model";
import { Template } from "../../text/Template";
import { Units } from "../../text/Units";

export interface ProgressReadoutText {
    /** Accessible name of the bar. */
    progressLabel: string;
    /** Template with a `{time}` slot, for example "about {time} left". */
    timeLeft: string;
    /** Between the figures of the line ("1.28 / 2.06 GB · 38 MB/s"), spaces included. */
    separator: string;
}

export interface ProgressReadoutProps {
    /** Caps line above the percent, built by the caller ("Step 3 of 6"). */
    eyebrow: string;
    progress: InstallProgress;
    text: ProgressReadoutText;
}

export class ProgressReadoutRules {
    /** `1.28 / 2.06 GB · 38 MB/s · about 20 s left`. Parts that are not known yet are left out. */
    static line(progress: InstallProgress, text: ProgressReadoutText): string {
        const parts = [Units.en.sizeOf(progress.done, progress.total)];
        if (progress.rate !== undefined) parts.push(Units.en.rate(progress.rate));
        if (progress.remaining !== undefined) {
            parts.push(Template.fill(text.timeLeft, { time: Units.en.duration(progress.remaining) }));
        }
        return parts.join(text.separator);
    }
}
