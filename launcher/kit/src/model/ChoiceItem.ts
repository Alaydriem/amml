/** One row of a single-choice list with an optional second line and right-hand note. */
export interface ChoiceItem {
    id: string;
    title: string;
    detail?: string;
    meta?: string;
}
