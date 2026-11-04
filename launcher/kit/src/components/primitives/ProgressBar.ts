export interface ProgressBarProps {
    /** Progress from 0 to 100. Values outside the range are clamped. */
    value: number;
    /** Accessible name. */
    label: string;
}

export class ProgressBarRules {
    /** Clamps to 0..100 and maps a non-number to 0. */
    static clamp(value: number): number {
        if (!Number.isFinite(value)) return 0;
        return Math.min(100, Math.max(0, value));
    }
}
