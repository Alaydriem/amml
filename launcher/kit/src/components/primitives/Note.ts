import type { Snippet } from "svelte";

export interface NoteProps {
    /** The note. Wrap emphasis in `<strong>`; wrap paths and codes in `<code>`. */
    children: Snippet;
}
