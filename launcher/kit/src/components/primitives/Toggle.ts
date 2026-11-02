/** `md` is the settings switch (48 × 28), `sm` the sidebar switch (40 × 24). */
export type ToggleSize = "md" | "sm";

export interface ToggleProps {
    /** Shown state. The switch never changes it: the parent answers `onchange` by passing a new value. */
    checked?: boolean;
    /** Accessible name, used when no visible caption names the switch. */
    label: string;
    size?: ToggleSize;
    disabled?: boolean;
    /** Id of the button, so a `<label for>` can point at it. */
    id?: string;
    /** Id of a visible caption that names the switch. It takes the place of `label`. */
    labelledby?: string;
    /** Called with the state the user asks for. */
    onchange?: (next: boolean) => void;
}
