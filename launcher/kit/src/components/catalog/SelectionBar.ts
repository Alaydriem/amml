import type { ModCard } from "../../model";
import { Template, type Plural, type TextPart } from "../../text/Template";
import { Units } from "../../text/Units";

export interface SelectionBarText {
    /** Template with `{count}` and `{names}` slots; `**…**` marks the strong part: "**{count} selected** · {names}". */
    summary: string;
    /** Between the names: ", ". */
    nameSeparator: string;
    clear: string;
    /** Caps button: "Install 1 mod" and "Install 2 mods". */
    install: Plural;
    /** Caps button while nothing is ticked: "Install". */
    installNone: string;
    /** Shown in place of the summary while nothing is ticked. */
    hint: string;
}

export interface SelectionBarProps {
    /** The ticked mods. While the list is empty the bar shows the hint and a disabled Install button. */
    selected: ModCard[];
    text: SelectionBarText;
    onclear?: () => void;
    oninstall?: () => void;
}

export class SelectionBarRules {
    static count(cards: readonly ModCard[]): number {
        return cards.length;
    }

    /** Sum of the download sizes in bytes. */
    static totalBytes(cards: readonly ModCard[]): number {
        return cards.reduce((sum, card) => sum + card.size, 0);
    }

    static totalSize(cards: readonly ModCard[]): string {
        return Units.en.size(SelectionBarRules.totalBytes(cards));
    }

    static names(cards: readonly ModCard[], separator: string): string {
        return cards.map((card) => card.title).join(separator);
    }

    static summary(cards: readonly ModCard[], text: SelectionBarText): TextPart[] {
        return Template.parts(text.summary, {
            count: cards.length,
            names: SelectionBarRules.names(cards, text.nameSeparator),
        });
    }

    static installLabel(cards: readonly ModCard[], text: SelectionBarText): string {
        if (cards.length === 0) return text.installNone;
        return Template.plural(text.install, cards.length);
    }
}
