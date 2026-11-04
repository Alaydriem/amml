import type { Snippet } from "svelte";

export interface AppFrameProps {
    /** The title bar: a `TitleBar`. */
    titlebar: Snippet;
    /** A banner drawn between the title bar and the page. It does not scroll with the page. */
    banner?: Snippet;
    /** The page. */
    children: Snippet;
    /** The status bar: a `StatusBar`. Omit on a page with its own footer. */
    statusbar?: Snippet;
    /** The body scrolls when its content is taller than the window. Default false: the page lays out its own panes. */
    scroll?: boolean;
}
