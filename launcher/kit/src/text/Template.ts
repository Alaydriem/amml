/** A text with a singular and a plural form. Both may use `{count}`. */
export interface Plural {
    one: string;
    other: string;
}

/** One run of a template: plain text, or text the template marks as strong with `**…**`. */
export interface TextPart {
    text: string;
    strong: boolean;
}

/** Fills `{name}` slots in a text template. An unknown slot stays visible so a missing value shows on screen. */
export class Template {
    static fill(template: string, values: Record<string, string | number>): string {
        return template.replace(/\{(\w+)\}/g, (slot, name: string) =>
            Object.hasOwn(values, name) ? String(values[name]) : slot,
        );
    }

    /** Picks the `one` or `other` form by count, then fills it with `count` and `values`. */
    static plural(forms: Plural, count: number, values: Record<string, string | number> = {}): string {
        return Template.fill(count === 1 ? forms.one : forms.other, { count, ...values });
    }

    /**
     * Splits a template at its `**…**` marks, then fills the slots of each run. The marks are read from the template
     * only, so a value that contains `**` is shown as it is. A mark with no partner stays in the text. Empty runs
     * are dropped.
     */
    static parts(template: string, values: Record<string, string | number> = {}): TextPart[] {
        const parts: TextPart[] = [];
        let last = 0;
        const push = (raw: string, strong: boolean) => {
            if (raw === "") return;
            parts.push({ text: Template.fill(raw, values), strong });
        };
        for (const match of template.matchAll(/\*\*(.+?)\*\*/g)) {
            push(template.slice(last, match.index), false);
            push(match[1] ?? "", true);
            last = match.index + match[0].length;
        }
        push(template.slice(last), false);
        return parts;
    }
}
