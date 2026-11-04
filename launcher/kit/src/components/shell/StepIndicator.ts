export interface StepIndicatorProps {
    /** Number of steps (bars). */
    total: number;
    /** Current step, from 1. Bars up to and including it are lit. */
    current: number;
    /** Template for the caption, with `{current}` and `{total}` slots ("Setup · {current} of {total}"). */
    template: string;
}
