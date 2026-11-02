import type { Snippet } from "svelte";
import type { DataMode } from "../../model";

export interface NewInstanceFormText {
    nameLabel: string;
    /** Template with a `{folder}` slot, for example "Folder: {folder}". */
    folderHint: string;
    worldsLabel: string;
    shared: string;
    own: string;
    cancel: string;
    create: string;
}

/** What the form hands back when Create is pressed. */
export interface NewInstanceRequest {
    /** Trimmed. */
    name: string;
    dataMode: DataMode;
}

export interface NewInstanceFormProps {
    /** Text in the name field. The form never changes it: the parent answers `onnamechange` with a new value. */
    name: string;
    dataMode: DataMode;
    /** Names of existing instances. A name that matches one of them cannot be created. */
    takenNames: string[];
    /** Folder path the app derives from the name, shown in the hint. */
    folder: string;
    /** Set false to hold Create back for a reason the form does not know (no version chosen). */
    canSubmit?: boolean;
    text: NewInstanceFormText;
    /** The summary callout body. */
    summary: Snippet;
    onnamechange?: (name: string) => void;
    ondatamodechange?: (dataMode: DataMode) => void;
    oncreate?: (request: NewInstanceRequest) => void;
    oncancel?: () => void;
}

export class NewInstanceFormRules {
    private static readonly FORBIDDEN = /[<>:"/\\|?*\u0000-\u001f]/;

    private static readonly RESERVED = /^(con|prn|aux|nul|com[1-9]|lpt[1-9])$/i;

    /**
     * The name as Windows sees the folder: outer spaces are cut, and so are trailing dots and spaces, which
     * Windows drops from a folder name.
     */
    static normalize(name: string): string {
        return name.trim().replace(/[. ]+$/, "");
    }

    /**
     * True when Windows accepts the name as a folder name: it is not empty, has none of the characters
     * `<>:"/\|?*` or a control character, and its stem (the part before the first dot) is not a reserved device
     * name such as CON, NUL, COM1 or LPT9.
     */
    static isFolderName(name: string): boolean {
        const normal = NewInstanceFormRules.normalize(name);
        if (normal === "") return false;
        if (NewInstanceFormRules.FORBIDDEN.test(normal)) return false;
        const stem = (normal.split(".")[0] ?? "").trimEnd();
        return !NewInstanceFormRules.RESERVED.test(stem);
    }

    /**
     * Create is allowed when the name is a valid folder name and no existing instance has it. Windows folder names
     * ignore case and trailing dots and spaces, so the comparison does too.
     */
    static canCreate(name: string, takenNames: string[]): boolean {
        if (!NewInstanceFormRules.isFolderName(name)) return false;
        const normal = NewInstanceFormRules.normalize(name).toLowerCase();
        return !takenNames.some((taken) => NewInstanceFormRules.normalize(taken).toLowerCase() === normal);
    }
}
