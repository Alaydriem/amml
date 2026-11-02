import type { AddModContext } from "../../model";
import { Template, type TextPart } from "../../text/Template";
import type { ContextChipPart, ContextChipText } from "../primitives/ContextChip";

export interface AddModHeaderText {
    back: string;
    /** Template with a `{name}` slot; `**…**` marks the strong part: "Add mods to **{name}**". */
    title: string;
    /** The labels of the chip pairs. */
    labels: { game: string; amml: string; api: string };
    /** The words of the chip. */
    chip: ContextChipText;
}

export interface AddModHeaderProps {
    /** Name of the instance, shown strong in the title. */
    name: string;
    context: AddModContext;
    text: AddModHeaderText;
    onback?: () => void;
}

export class AddModHeaderRules {
    static title(name: string, text: AddModHeaderText): TextPart[] {
        return Template.parts(text.title, { name });
    }

    /** The game, AMML and api pairs of the chip, in that order. */
    static parts(context: AddModContext, text: AddModHeaderText): ContextChipPart[] {
        return [
            { id: "game", label: text.labels.game, value: context.game },
            { id: "amml", label: text.labels.amml, value: context.amml },
            { id: "api", label: text.labels.api, value: context.api, tone: "ok" },
        ];
    }
}
