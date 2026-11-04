import type { Snippet } from "svelte";

export interface SettingsLayoutProps {
    /** The side navigation: a `SettingsNav`. */
    nav: Snippet;
    /** The open section. Stack cards and notes; the layout spaces them 22 px apart. */
    children: Snippet;
}
