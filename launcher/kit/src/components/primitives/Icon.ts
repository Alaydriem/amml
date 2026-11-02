/** One drawing of an icon on a 24 × 24 grid. Paths are copied from the Paper page "Launcher v1". */
export interface IconShape {
    /** Stroked paths (2 px, round caps and joins). */
    strokes?: string[];
    /** Filled paths. */
    fills?: string[];
    /** Stroked circles: cx, cy, r. */
    circles?: [number, number, number][];
    /** Filled circles: cx, cy, r. */
    dots?: [number, number, number][];
}

export const ICONS = {
    plus: { strokes: ["M12 5v14M5 12h14"] },
    "arrow-up": { strokes: ["M12 19V5M5 12l7-7 7 7"] },
    refresh: { strokes: ["M21 12a9 9 0 1 1-2.64-6.36", "M21 3v6h-6"] },
    settings: {
        circles: [[12, 12, 3]],
        strokes: [
            "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z",
        ],
    },
    more: {
        dots: [
            [12, 5, 1.8],
            [12, 12, 1.8],
            [12, 19, 1.8],
        ],
    },
    play: { fills: ["M6 4l14 8-14 8z"] },
    check: { strokes: ["M5 12l5 5L20 7"] },
    close: { strokes: ["M6 6l12 12M18 6L6 18"] },
    "chevron-left": { strokes: ["M15 18l-6-6 6-6"] },
    "chevron-down": { strokes: ["M6 9l6 6 6-6"] },
    search: { circles: [[11, 11, 7]], strokes: ["M20 20l-4-4"] },
    link: {
        strokes: [
            "M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1",
            "M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1",
        ],
    },
    shield: { strokes: ["M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z"] },
    "alert-circle": { circles: [[12, 12, 9]], strokes: ["M12 7v6M12 16.5v.5"] },
    "alert-triangle": { strokes: ["M12 3l10 18H2z", "M12 10v5M12 18v.5"] },
    github: {
        fills: [
            "M12 .5C5.65.5.5 5.65.5 12a11.5 11.5 0 0 0 7.86 10.92c.58.1.79-.25.79-.56v-2c-3.2.7-3.87-1.37-3.87-1.37-.52-1.33-1.28-1.69-1.28-1.69-1.05-.71.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.19-3.1-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.17 1.18a11 11 0 0 1 5.77 0c2.2-1.49 3.17-1.18 3.17-1.18.63 1.59.23 2.76.11 3.05.74.81 1.19 1.84 1.19 3.1 0 4.42-2.69 5.39-5.25 5.68.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5z",
        ],
    },
} satisfies Record<string, IconShape>;

export type IconName = keyof typeof ICONS;

export interface IconProps {
    name: IconName;
    /** Rendered size in px. */
    size?: number;
    /** Stroke width on the 24 px grid. */
    weight?: number;
    /** Accessible name. Without it the icon is decorative and hidden from screen readers. */
    label?: string;
}
