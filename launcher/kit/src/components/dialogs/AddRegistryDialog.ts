export interface AddRegistryDialogText {
    title: string;
    /** Caption above the field; it also names the field. */
    fieldLabel: string;
    placeholder: string;
    /** The note under the field while the address does not start with `https://`. */
    invalid: string;
    add: string;
    cancel: string;
}

export interface AddRegistryDialogProps {
    open?: boolean;
    inline?: boolean;
    text: AddRegistryDialogText;
    /** The address the user typed. The field shows it and never changes it on its own. */
    value: string;
    /** Why the app refused the address, already in the user's language. Add stays disabled while it is set. */
    problem?: string;
    oninput?: (next: string) => void;
    /** Called with the trimmed address. */
    onadd?: (url: string) => void;
    oncancel?: () => void;
}

export class AddRegistryDialogRules {
    static readonly SCHEME = "https://";

    /** True when the trimmed address starts with `https://` and the app has no problem with it. */
    static canAdd(value: string, problem?: string): boolean {
        return !problem && value.trim().startsWith(AddRegistryDialogRules.SCHEME);
    }

    /** True when the field has text that is not an https address. */
    static isInvalid(value: string): boolean {
        const text = value.trim();
        return text !== "" && !text.startsWith(AddRegistryDialogRules.SCHEME);
    }
}
