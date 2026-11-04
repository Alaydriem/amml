import type { IconName } from "./Icon";

export interface IconButtonProps {
    icon: IconName;
    /** Accessible name. There is no visible text, so it is required. */
    label: string;
    /** Icon size in px. */
    iconSize?: number;
    disabled?: boolean;
    /** Set by a button that opens a menu. */
    haspopup?: "menu";
    expanded?: boolean;
    onclick?: () => void;
}
