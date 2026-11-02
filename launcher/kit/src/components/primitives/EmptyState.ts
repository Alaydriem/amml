import type { Snippet } from "svelte";
import type { IconName } from "./Icon";

export interface EmptyStateProps {
    /** Icon above the title. Without it the state draws the brand mark. */
    icon?: IconName;
    /** The heading ("No instances yet."). */
    title: string;
    /** The explanation under the title. */
    body?: Snippet;
    /** Buttons under the explanation. */
    actions?: Snippet;
}
