import { RovingFocus } from "./RovingFocus";

export interface MenuItem {
    id: string;
    label: string;
    /** `fault` draws the item in the fault colour (Delete, Remove). */
    tone?: "fault";
    disabled?: boolean;
    /** Draws a divider above the item. */
    separated?: boolean;
}

export interface MenuProps {
    items: MenuItem[];
    /** Accessible name of the three-dots button. */
    label: string;
    onselect?: (id: string) => void;
}

export class MenuRules {
    /** Index of the item that takes focus when the menu opens: the first enabled one, or -1. */
    static first(items: MenuItem[]): number {
        return RovingFocus.next(items.length, -1, 1, (index) => items[index]?.disabled === true);
    }

    /** Index to focus after a key press with `current` focused (-1 for none), or `null` when the key does nothing. */
    static target(items: MenuItem[], key: string, current: number): number | null {
        return RovingFocus.target(key, current, items.length, {
            axis: "vertical",
            disabled: (index) => items[index]?.disabled === true,
        });
    }

    /** True when a pointer press landed outside the menu. */
    static isOutside(root: HTMLElement | undefined, target: EventTarget | null): boolean {
        return root !== undefined && target instanceof Node && !root.contains(target);
    }
}
