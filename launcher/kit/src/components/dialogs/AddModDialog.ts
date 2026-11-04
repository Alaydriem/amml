import type { AddModSuggestion } from "../../model";

export interface AddModDialogText {
    /** The title; `{name}` is the instance. */
    title: string;
    /** Caption above the field; it also names the field. */
    fieldLabel: string;
    /** Warning for an input no registry lists. `**…**` marks the lead sentence. */
    unofficial: string;
    /** Caption above the suggestions. */
    suggestionsLabel: string;
    add: string;
    cancel: string;
    install: string;
}

export interface AddModDialogProps {
    open?: boolean;
    inline?: boolean;
    text: AddModDialogText;
    /** Values for the slots of `title`. */
    values?: Record<string, string | number>;
    /** The id or zip URL the user typed. The field shows it and never changes it on its own. */
    value: string;
    /** Ids the registries list. An input that is not one of them installs as unofficial. */
    knownIds: string[];
    suggestions: AddModSuggestion[];
    oninput?: (next: string) => void;
    onadd?: (id: string) => void;
    /** Called with the trimmed input when the user installs it as unofficial. */
    oninstallurl?: (url: string) => void;
    oncancel?: () => void;
}

export class AddModDialogRules {
    /** True when the input is not empty and no registry lists it. */
    static isUnofficial(input: string, knownIds: string[]): boolean {
        const text = input.trim();
        return text !== "" && !knownIds.includes(text);
    }
}
