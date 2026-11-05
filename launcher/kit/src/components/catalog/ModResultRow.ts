import type { ModCard, ModCardState } from "../../model";

export interface ModResultRowText {
    installed: string;
    update: string;
    /** Template with a `{title}` slot: the accessible name of the row's checkbox. */
    select: string;
}

/** The row is a list item: place it inside a `<ul>`. */
export interface ModResultRowProps {
    card: ModCard;
    /** The row is ticked. */
    checked?: boolean;
    /** The row is the one shown in the detail pane. */
    active?: boolean;
    /** Marks the first loaded row of a group, which the list scrolls to when earlier rows are missing. */
    anchor?: boolean;
    text: ModResultRowText;
    /** Called with the id and the state the user asks for. */
    onchange?: (id: string, checked: boolean) => void;
    /** Called when the row's main button, or any other part of the row, is pressed. */
    onactivate?: (id: string) => void;
}

export class ModResultRowRules {
    /** An installed mod cannot be ticked again, and a mod that needs a newer AMML cannot be ticked. An update row can. */
    static selectable(state: ModCardState): boolean {
        return state !== "installed" && state !== "needsAmml";
    }

    /** The selection after a row is ticked or cleared; ids keep their order and appear once. */
    static toggle(selected: readonly string[], id: string, checked: boolean): string[] {
        const rest = selected.filter((entry) => entry !== id);
        return checked ? [...rest, id] : rest;
    }
}
