export interface SearchFieldProps {
    /** Shown text. The field never changes it: the parent answers `oninput` by passing a new value. */
    value?: string;
    /** Accessible name. */
    label: string;
    placeholder?: string;
    disabled?: boolean;
    oninput?: (next: string) => void;
    /** Called with the text when Enter is pressed. */
    onsubmit?: (value: string) => void;
}
