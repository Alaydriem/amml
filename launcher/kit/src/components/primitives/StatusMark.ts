/**
 * The small square that leads a check or step row. `ok` a green tick, `warn` an amber exclamation mark, `failed` a
 * red cross, `active` the current step, `pending` an empty outline.
 */
export type StatusMarkState = "ok" | "warn" | "failed" | "active" | "pending";

export interface StatusMarkProps {
    state: StatusMarkState;
    /** Square size in px: 22 on the setup and install rows, 20 in the URL check list. */
    size?: 20 | 22;
    /**
     * What the mark says ("Done", "Failed", "Waiting"), read by assistive technology and not drawn. Without it the
     * mark is decorative and the row text carries the state.
     */
    label?: string;
}
