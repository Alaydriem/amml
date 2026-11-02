/** One labelled value, as in the bug report panel (artboard 07e). */
export interface KeyValue {
    id: string;
    label: string;
    value: string;
    /** When set the value is a link or action. */
    href?: string;
}
