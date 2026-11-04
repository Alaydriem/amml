import { unitsEn } from "./en/units";
import { Template } from "./Template";

/**
 * The words and the locale of sizes, rates and durations. Each unit is a template with an `{n}` slot; the compound
 * duration has `{hours}` and `{minutes}`.
 */
export interface UnitsText {
    gigabytes: string;
    megabytes: string;
    kilobytes: string;
    /** Transfer rate in megabytes. */
    megabytesPerSecond: string;
    seconds: string;
    minutes: string;
    hours: string;
    hoursMinutes: string;
    /** BCP 47 tag used to group digits in counts. */
    locale: string;
}

export type SizeUnit = "gb" | "mb" | "kb";

export interface SizeOptions {
    /** Forces a unit. Without it the unit is chosen after rounding. */
    unit?: SizeUnit;
    /** Decimals. Default 2 for GB, 1 for MB, 0 for KB. */
    digits?: number;
    /** Drops trailing zeros of the number (`1.90` becomes `1.9`, `3.0` becomes `3`). */
    trim?: boolean;
}

/** Formats sizes, rates, durations and counts with the words of a `UnitsText`. */
export class Units {
    static readonly GB = 1_000_000_000;
    static readonly MB = 1_000_000;
    static readonly KB = 1_000;

    /** Sizes below this many bytes (0.1 MB) show in KB. */
    static readonly KB_LIMIT = 100_000;

    private static readonly DIGITS: Record<SizeUnit, number> = { gb: 2, mb: 1, kb: 0 };
    private static readonly BYTES: Record<SizeUnit, number> = { gb: Units.GB, mb: Units.MB, kb: Units.KB };

    /** The English words, for callers that have no catalog yet. */
    static readonly en: Units = new Units(unitsEn);

    readonly text: UnitsText;

    constructor(text: UnitsText) {
        this.text = text;
    }

    /** Whole percent, clamped to 0..100. */
    static percent(done: number, total: number): number {
        if (total <= 0) return 0;
        return Math.min(100, Math.max(0, Math.floor((done / total) * 100)));
    }

    /**
     * `2.06 GB`, `0.6 MB`, `45 KB`. The unit is chosen after the number is rounded, so a size just under 1 GB reads
     * `1.00 GB` and never `1000.0 MB`. Below 0.1 MB the unit is KB.
     */
    size(bytes: number, options: SizeOptions = {}): string {
        const unit = options.unit ?? this.unitFor(bytes, options.digits);
        return this.withUnit(this.number(bytes, unit, options), unit);
    }

    /** The number and the unit word of a size apart, for a big figure with a small unit: `{ value: "1.9", unit: "GB" }`. */
    sizeParts(bytes: number, options: SizeOptions = {}): { value: string; unit: string } {
        const unit = options.unit ?? this.unitFor(bytes, options.digits);
        const template = { gb: this.text.gigabytes, mb: this.text.megabytes, kb: this.text.kilobytes }[unit];
        return { value: this.number(bytes, unit, options), unit: template.replace("{n}", "").trim() };
    }

    /** `1.28 / 2.06 GB`: both values in the unit of the total. */
    sizeOf(done: number, total: number, options: Omit<SizeOptions, "unit"> = {}): string {
        const unit = this.unitFor(total, options.digits);
        const both = `${this.number(done, unit, options)} / ${this.number(total, unit, options)}`;
        return this.withUnit(both, unit);
    }

    /** `38 MB/s` from 1 MB/s up, `0.4 MB/s` below. */
    rate(bytesPerSecond: number): string {
        const value = Math.max(0, bytesPerSecond) / Units.MB;
        const n = value < 1 ? value.toFixed(1) : String(Math.round(value));
        return Template.fill(this.text.megabytesPerSecond, { n });
    }

    /** `20 s`, `3 min`, `1 h 5 min`. */
    duration(seconds: number): string {
        const s = Math.max(0, Math.round(seconds));
        if (s < 60) return Template.fill(this.text.seconds, { n: s });
        const minutes = Math.round(s / 60);
        if (minutes < 60) return Template.fill(this.text.minutes, { n: minutes });
        const hours = Math.floor(minutes / 60);
        const rest = minutes % 60;
        if (rest === 0) return Template.fill(this.text.hours, { n: hours });
        return Template.fill(this.text.hoursMinutes, { hours, minutes: rest });
    }

    /** `1,412`, grouped by the locale. */
    count(n: number): string {
        return n.toLocaleString(this.text.locale);
    }

    private unitFor(bytes: number, digits?: number): SizeUnit {
        const abs = Math.max(0, bytes);
        if (abs >= Units.GB) return "gb";
        if (abs < Units.KB_LIMIT) return "kb";
        // A value that rounds to 1000 MB belongs to the next unit.
        const rounded = Number((abs / Units.MB).toFixed(digits ?? Units.DIGITS.mb));
        return rounded >= 1000 ? "gb" : "mb";
    }

    private number(bytes: number, unit: SizeUnit, options: SizeOptions): string {
        const digits = options.digits ?? Units.DIGITS[unit];
        const fixed = (Math.max(0, bytes) / Units.BYTES[unit]).toFixed(digits);
        return options.trim ? String(Number(fixed)) : fixed;
    }

    private withUnit(n: string, unit: SizeUnit): string {
        const template = { gb: this.text.gigabytes, mb: this.text.megabytes, kb: this.text.kilobytes }[unit];
        return Template.fill(template, { n });
    }
}
