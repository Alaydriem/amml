import type { AdoptedInstance } from "../../model";
import { Template, type Plural } from "../../text/Template";

export interface AdoptedInstanceText {
    /** Template with `{amml}` and `{mods}` slots, for example "AMML {amml} · {mods}". */
    meta: string;
    /** Shown in place of the version when it is unknown. */
    unknownVersion: string;
    /** Mods count, for example "{count} mod" and "{count} mods". */
    mods: Plural;
}

export interface AdoptedInstanceRowProps {
    instance: AdoptedInstance;
    text: AdoptedInstanceText;
}

export class AdoptedInstanceRowRules {
    /** The right-hand note of the row. */
    static meta(instance: AdoptedInstance, text: AdoptedInstanceText): string {
        return Template.fill(text.meta, {
            amml: instance.ammlVersion ?? text.unknownVersion,
            mods: Template.plural(text.mods, instance.modCount),
        });
    }
}
