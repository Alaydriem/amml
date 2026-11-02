export interface TextFieldProps {
    /** Shown text. The field never changes it: the parent answers `oninput` by passing a new value. */
    value?: string;
    /** Accessible name, used when no visible caption names the field. */
    label: string;
    placeholder?: string;
    /** Monospace text, for ids, paths and URLs. */
    mono?: boolean;
    disabled?: boolean;
    readonly?: boolean;
    /** Id of the input, so a `<label for>` can point at it. */
    id?: string;
    /** Id of a visible caption that names the field. It takes the place of `label`. */
    labelledby?: string;
    oninput?: (next: string) => void;
    /** Called with the text when Enter is pressed in the field. */
    onsubmit?: (value: string) => void;
}
