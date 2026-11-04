import type { Option } from "../../model";

export interface SelectProps {
    options: Option[];
    /** Id of the selected option. The control never changes it: the parent answers `onchange` with a new value. */
    value?: string;
    /** Accessible name. */
    label: string;
    /** Drops the 150 px minimum width (the sort control above the mod list). */
    compact?: boolean;
    disabled?: boolean;
    /** Called with the id the user picked. */
    onchange?: (id: string) => void;
}
