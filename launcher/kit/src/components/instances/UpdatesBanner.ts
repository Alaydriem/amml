import { Template, type Plural, type TextPart } from "../../text/Template";

export interface UpdatesBannerText {
    /**
     * Template with `{updates}` and `{instances}` slots. Mark the lead with `**…**`, for example
     * "**{updates} available** in {instances}. Nothing is installed until you ask."
     */
    message: string;
    /** "{count} update" and "{count} updates". */
    updates: Plural;
    /** "{count} instance" and "{count} instances". */
    instances: Plural;
    /** Label of the trailing action ("Review"). */
    review: string;
}

export interface UpdatesBannerProps {
    /** Updates in all instances. */
    updates: number;
    /** Instances that have at least one update. */
    instances: number;
    text: UpdatesBannerText;
    onreview?: () => void;
}

export class UpdatesBannerRules {
    /** The message split into plain and strong runs, with both counts in their plural forms. */
    static parts(updates: number, instances: number, text: UpdatesBannerText): TextPart[] {
        return Template.parts(text.message, {
            updates: Template.plural(text.updates, updates),
            instances: Template.plural(text.instances, instances),
        });
    }
}
