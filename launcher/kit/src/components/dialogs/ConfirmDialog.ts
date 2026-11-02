export interface ConfirmDialogText {
    /** Warn-tone mono caps line above the title ("Needs your OK"). */
    eyebrow?: string;
    /** The title. Slots such as `{name}` are filled from `values`. */
    title: string;
    /** The question. `**…**` marks emphasis; slots are filled from `values`. */
    body: string;
    /** Faint note under the body and the path. */
    note?: string;
    cancel: string;
    confirm: string;
}

export interface ConfirmDialogProps {
    /** Shown when true. With `inline` the dialog is always drawn. */
    open?: boolean;
    /** Draws the card in the flow with no scrim (gallery sheets). */
    inline?: boolean;
    text: ConfirmDialogText;
    /** Values for the slots of `title`, `body` and `note`. */
    values?: Record<string, string | number>;
    /** Mono box under the body, for a folder path. */
    path?: string;
    /** `primary` for a step forward, `danger` for a delete. */
    confirmVariant?: "primary" | "danger";
    onconfirm?: () => void;
    /** Called by the cancel button, Escape and a click on the scrim. */
    oncancel?: () => void;
}
