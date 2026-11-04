import type { Instance } from "../../model";
import { Template, type Plural } from "../../text/Template";
import type { MenuItem } from "../primitives/Menu";

export interface InstanceRowText {
    /** Column labels. */
    game: string;
    amml: string;
    mods: string;
    launch: string;
    running: string;
    upToDate: string;
    /** Update count, for example "{count} update" and "{count} updates". */
    updates: Plural;
    /** Template with `{data}` and `{activity}` slots, for example "{data} · {activity}". */
    activityLine: string;
    dataShared: string;
    dataOwn: string;
    /** Accessible name of the row menu button. */
    menuLabel: string;
}

export interface InstanceRowProps {
    instance: Instance;
    text: InstanceRowText;
    /** Items of the row menu. */
    menuItems: MenuItem[];
    /** Draws the Launch button filled. The list sets it on its first row. */
    emphasis?: boolean;
    /** Not called for a running instance: the row shows a status in place of the Launch button. */
    onlaunch?: (id: string) => void;
    /** The name and tile were pressed: open the instance. */
    onopen?: (id: string) => void;
    onmenu?: (id: string, itemId: string) => void;
}

export class InstanceRowRules {
    /** "Shared data · last played 2 h ago". */
    static activityLine(instance: Instance, text: InstanceRowText): string {
        return Template.fill(text.activityLine, {
            data: instance.dataMode === "shared" ? text.dataShared : text.dataOwn,
            activity: instance.activity,
        });
    }

    /** The update pill text, or `null` when the instance is up to date. */
    static updateLabel(instance: Instance, text: InstanceRowText): string | null {
        return instance.updateCount > 0 ? Template.plural(text.updates, instance.updateCount) : null;
    }
}
