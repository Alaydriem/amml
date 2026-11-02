/**
 * `check` is the filled brand square with a tick (a ticked row in the add-mod list). `marker` is the outlined
 * square with an inner brand-lift square (the selected row of a single-choice list).
 */
export type CheckboxVariant = "check" | "marker";

interface CheckboxBase {
    /** Shown state. The box never changes it: the parent answers `onchange` by passing a new value. */
    checked?: boolean;
    variant?: CheckboxVariant;
}

/** A button the user can press. */
export interface CheckboxControlProps extends CheckboxBase {
    decorative?: false;
    /** Accessible name. */
    label: string;
    disabled?: boolean;
    /** Called with the state the user asks for. */
    onchange?: (next: boolean) => void;
}

/** A mark inside a row that is itself the control: hidden from assistive technology, not focusable. */
export interface CheckboxDecorativeProps extends CheckboxBase {
    decorative: true;
    label?: never;
    disabled?: never;
    onchange?: never;
}

export type CheckboxProps = CheckboxControlProps | CheckboxDecorativeProps;
