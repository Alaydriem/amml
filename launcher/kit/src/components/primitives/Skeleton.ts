/** Corner of a skeleton block: `xs` 4 px, `sm` 6 px, `seg` 7 px (the tile). */
export type SkeletonRadius = "xs" | "sm" | "seg";

export interface SkeletonProps {
    /** CSS width of the block ("38%", "28px"). Default "100%". */
    width?: string;
    /** Height in px. Default 12. */
    height?: number;
    /** `strong` is the title bar (line-2), `soft` the second line and the tile (line), `panel` the detail pane. */
    tone?: "strong" | "soft" | "panel";
    radius?: SkeletonRadius;
}
