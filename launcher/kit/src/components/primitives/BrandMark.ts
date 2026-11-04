/** `sm` is the title bar mark (26 × 30), `md` the empty-state mark (52 × 60), `lg` the about page mark (64 × 74). */
export type BrandMarkSize = "sm" | "md" | "lg";

export interface BrandMarkProps {
    size?: BrandMarkSize;
    /** Accessible name. Without it the mark is decorative. */
    label?: string;
}

export class BrandMarkRules {
    /** Pixel width and height, and the stroke width on the 26 × 30 drawing, for each size. */
    static readonly SIZES: Record<BrandMarkSize, { width: number; height: number; stroke: number }> = {
        sm: { width: 26, height: 30, stroke: 3 },
        md: { width: 52, height: 60, stroke: 2.4 },
        lg: { width: 64, height: 74, stroke: 2.4 },
    };

    /** The six hexagon edges, in drawing order, with the spectrum token each one uses. */
    static readonly SEGMENTS: readonly { d: string; color: string }[] = [
        { d: "M13 2 L2 8.5", color: "var(--color-spectrum-03)" },
        { d: "M13 2 L24 8.5", color: "var(--color-spectrum-05)" },
        { d: "M24 8.5 L24 21.5", color: "var(--color-spectrum-07)" },
        { d: "M24 21.5 L13 28", color: "var(--color-spectrum-09)" },
        { d: "M13 28 L2 21.5", color: "var(--color-spectrum-10)" },
        { d: "M2 21.5 L2 8.5", color: "var(--color-spectrum-01)" },
    ];
}
