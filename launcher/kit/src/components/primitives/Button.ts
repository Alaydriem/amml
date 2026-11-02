import type { Snippet } from "svelte";
import type { IconName } from "./Icon";

/**
 * `primary` filled brand; `secondary` outlined on a raised ground; `ghost` text only; `danger` fault text on a
 * fault tint; `warn` warn outline (install unofficial); `launch` the caps launch button of the instance list;
 * `running` the caps outlined state of a running instance; `link` text with no box, coloured by `tone`.
 */
export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "warn" | "launch" | "running" | "link";

export type ButtonSize = "sm" | "md" | "lg";

/** Colour of a `link` button. */
export type ButtonTone = "brand" | "fault" | "dim";

export interface ButtonProps {
    variant?: ButtonVariant;
    size?: ButtonSize;
    /** Colour of a `link` button. Default `brand`. Other variants ignore it. */
    tone?: ButtonTone;
    icon?: IconName;
    /**
     * Caps label with the 4 px corner of the wizard, form and selection-bar buttons. `launch` and `running` are
     * always caps. With `size` it sets the padding: sm 11/20, md 12/22, lg 14/26.
     */
    caps?: boolean;
    disabled?: boolean;
    /** Stretches to the parent's width. */
    block?: boolean;
    type?: "button" | "submit";
    onclick?: () => void;
    children: Snippet;
}
