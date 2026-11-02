import type { Instance } from "../../model";
import type { MenuItem } from "../primitives/Menu";
import type { InstanceRowText } from "./InstanceRow";

export interface InstanceListProps {
    instances: Instance[];
    text: InstanceRowText;
    /** Items of every row menu. */
    menuItems: MenuItem[];
    /** Accessible name of the list. */
    label: string;
    onlaunch?: (id: string) => void;
    onopen?: (id: string) => void;
    onmenu?: (id: string, itemId: string) => void;
}
