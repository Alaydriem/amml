import type { ChoiceItem } from "../../model";

export interface ChoiceListProps {
    /** Panel header ("Copy into shared data"), also the accessible name of the group. */
    heading: string;
    choices: ChoiceItem[];
    /** Id of the selected choice. The list never changes it: the parent answers `onchange` with a new value. */
    value?: string;
    /** Called with the id the user asks for, by click or arrow key. */
    onchange?: (id: string) => void;
}
