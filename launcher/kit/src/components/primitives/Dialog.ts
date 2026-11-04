import type { Snippet } from "svelte";

export interface DialogProps {
    /** Shown when true. With `inline` the dialog is always drawn. */
    open?: boolean;
    title: string;
    /** Mono caps line above the title ("Needs your OK"). */
    eyebrow?: string;
    /** `md` 392 px, `lg` 448 px (the add-a-mod form). */
    size?: "md" | "lg";
    /** Draws the card in the flow with no scrim and no focus capture (gallery sheets). */
    inline?: boolean;
    /** Called on Escape and on a click on the scrim. Only the top layer receives Escape. */
    onclose?: () => void;
    /** Body. Use `<strong>` for emphasis. */
    children: Snippet;
    /** Right-aligned buttons. */
    actions?: Snippet;
}

export class DialogFocus {
    static readonly SELECTOR =
        'button:not(:disabled), [href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])';

    /** Focusable elements inside the card, in DOM order. */
    static within(card: HTMLElement): HTMLElement[] {
        return Array.from(card.querySelectorAll<HTMLElement>(DialogFocus.SELECTOR));
    }

    /**
     * Element that takes focus when Tab would leave the card, or `null` to let the browser move. Focus outside the
     * card goes to the first control, or to the last for Shift+Tab.
     */
    static wrap(card: HTMLElement, active: Element | null, backwards: boolean): HTMLElement | null {
        const items = DialogFocus.within(card);
        const first = items[0];
        const last = items[items.length - 1];
        if (!first || !last) return card;
        if (!active || !card.contains(active)) return backwards ? last : first;
        if (backwards && (active === first || active === card)) return last;
        if (!backwards && active === last) return first;
        return null;
    }
}
