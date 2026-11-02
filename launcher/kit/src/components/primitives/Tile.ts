import type { Swatch } from "../../model";
import type { IconName } from "./Icon";

/**
 * `row` 28 px (mod list), `detail` 44 px (detail pane), `rail` 52 px (instance list), `header` 64 px. `dot` is the
 * bare colour square (14 px) that leads a compact row, with no frame.
 */
export type TileSize = "row" | "detail" | "rail" | "header" | "dot";

export interface TileProps {
    size?: TileSize;
    /** Spectrum colour of the inner square. */
    swatch?: Swatch;
    /** Draws this icon in place of the colour square. */
    icon?: IconName;
    /** Draws this image (a mod icon) in place of the colour square. */
    src?: string;
    /** Accessible name of an icon or image. Without it the tile is decorative. */
    label?: string;
}

export class TileRules {
    /** Pixel size of the frame and of the inner square for each size. */
    static readonly SIZES: Record<TileSize, { outer: number; inner: number }> = {
        row: { outer: 28, inner: 11 },
        detail: { outer: 44, inner: 18 },
        rail: { outer: 52, inner: 22 },
        header: { outer: 64, inner: 26 },
        dot: { outer: 14, inner: 14 },
    };

    /** The one rule from a swatch number to its colour: `var(--color-spectrum-03)`. */
    static swatchColor(swatch: Swatch): string {
        return `var(--color-spectrum-${String(swatch).padStart(2, "0")})`;
    }
}
