import type { Instance } from "../model";
import type { InstanceRowText } from "../components/instances/InstanceRow";
import type { UpdatesBannerText } from "../components/instances/UpdatesBanner";
import type { MenuItem } from "../components/primitives/Menu";

export interface InstancesScreenText {
    /** Template with a `{count}` slot, for example "Instances · {count}". */
    eyebrow: string;
    /** Title with the bold part marked by `**…**`, for example "Launch **AMML.**". */
    title: string;
    newInstance: string;
    banner: UpdatesBannerText;
    /** Accessible name of the list. */
    listLabel: string;
    /** The state with no instance. */
    empty: {
        title: string;
        body: string;
    };
    row: InstanceRowText;
}

export interface InstancesScreenProps {
    instances: Instance[];
    text: InstancesScreenText;
    /** Items of every row menu. */
    menuItems: MenuItem[];
    onnew?: () => void;
    /** The banner action was pressed. */
    onreview?: () => void;
    onlaunch?: (id: string) => void;
    onopen?: (id: string) => void;
    onmenu?: (id: string, itemId: string) => void;
}

export class InstancesScreenRules {
    /** Updates in all instances and the number of instances that have at least one. */
    static updateTotals(instances: Instance[]): { updates: number; instances: number } {
        let updates = 0;
        let affected = 0;
        for (const instance of instances) {
            updates += instance.updateCount;
            if (instance.updateCount > 0) affected += 1;
        }
        return { updates, instances: affected };
    }
}
