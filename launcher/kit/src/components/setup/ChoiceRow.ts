export interface ChoiceRowProps {
    id: string;
    title: string;
    /** Mono line under the title. */
    detail?: string;
    /** Right-hand mono note ("14 worlds · today"). */
    meta?: string;
    selected?: boolean;
    /** Whether the row is the tab stop of its group. */
    stop?: boolean;
    /** Called with the row id when the row is chosen. */
    onselect?: (id: string) => void;
    /** Key presses on the row, for the group's arrow-key movement. */
    onkeydown?: (event: KeyboardEvent) => void;
}
