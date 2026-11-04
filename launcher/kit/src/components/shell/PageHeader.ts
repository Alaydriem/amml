import type { Snippet } from "svelte";

export interface PageHeaderProps {
    /** Mono caps line above the title ("Instances · 3"). */
    eyebrow?: string;
    /** `page` is the 34 px title, `section` the 24 px title of the add-mod header. */
    size?: "page" | "section";
    /** The title. Wrap the bold part in `<strong>`: "Launch <strong>AMML.</strong>". */
    title: Snippet;
    /** Trailing buttons. */
    actions?: Snippet;
}
