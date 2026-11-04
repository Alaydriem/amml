import type { Option } from "../../model";

export interface SegmentedProps {
    options: Option[];
    /** Id of the selected option. The control never changes it: the parent answers `onchange` with a new value. */
    value?: string;
    /** Accessible name of the group. */
    label: string;
    disabled?: boolean;
    /** Called with the id the user asks for, by click or arrow key. */
    onchange?: (id: string) => void;
}
