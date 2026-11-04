export type RovingAxis = "horizontal" | "vertical" | "both";

export interface RovingOptions {
    /** Which arrow keys move. Default `both`. */
    axis?: RovingAxis;
    /** Items the arrows skip. */
    disabled?: (index: number) => boolean;
}

/**
 * Arrow-key movement over a row of items where one item is the tab stop: menus, segmented controls, radio
 * groups. The class decides the next index; the component moves focus and selection.
 */
export class RovingFocus {
    /**
     * Index of the next enabled item after `from` in the direction of `step`, wrapping. `from` of -1 means no item
     * has focus: a forward step lands on the first item and a backward step on the last. Returns -1 when every
     * item is disabled.
     */
    static next(count: number, from: number, step: 1 | -1, disabled?: (index: number) => boolean): number {
        if (count <= 0) return -1;
        let index = from < 0 && step < 0 ? count : from;
        for (let i = 0; i < count; i++) {
            index = (((index + step) % count) + count) % count;
            if (!disabled?.(index)) return index;
        }
        return -1;
    }

    /**
     * Index to move to after a key press, or `null` when the key does not move. Arrows wrap; Home and End go to the
     * first and last enabled item.
     */
    static target(key: string, from: number, count: number, options: RovingOptions = {}): number | null {
        const axis = options.axis ?? "both";
        const forward = ["ArrowDown", "ArrowRight"].filter((k) => RovingFocus.accepts(axis, k));
        const backward = ["ArrowUp", "ArrowLeft"].filter((k) => RovingFocus.accepts(axis, k));
        let result: number;
        if (forward.includes(key)) result = RovingFocus.next(count, from, 1, options.disabled);
        else if (backward.includes(key)) result = RovingFocus.next(count, from, -1, options.disabled);
        else if (key === "Home") result = RovingFocus.next(count, -1, 1, options.disabled);
        else if (key === "End") result = RovingFocus.next(count, -1, -1, options.disabled);
        else return null;
        return result < 0 ? null : result;
    }

    /** The tab stop: the selected item, or the first when nothing is selected. */
    static stop(selected: number): number {
        return selected >= 0 ? selected : 0;
    }

    private static accepts(axis: RovingAxis, key: string): boolean {
        if (axis === "both") return true;
        return axis === "vertical" ? key === "ArrowDown" || key === "ArrowUp" : key === "ArrowLeft" || key === "ArrowRight";
    }
}
