import type { Option } from "../../model";

export interface SettingsNavText {
    /** Label of the link above the list ("Instances"). */
    back: string;
    /** Accessible name of the navigation ("Settings sections"). */
    label: string;
}

export interface SettingsNavProps {
    text: SettingsNavText;
    items: Option[];
    /** Id of the open section. The nav never changes it: the parent answers `onchange` with a new value. */
    value: string;
    onback?: () => void;
    /** Called with the id of the section the user picked. */
    onchange?: (id: string) => void;
}
