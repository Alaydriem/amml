import type { Option } from "../../model";

export interface CatalogToolbarText {
    /** Accessible name of the search field. */
    searchLabel: string;
    /** Accessible name of the sort control. */
    sortLabel: string;
    sortOptions: Option[];
}

export interface CatalogToolbarProps {
    /** Shown search text. The field never changes it: the parent answers `onsearch` with a new value. */
    query?: string;
    /** Greyed text of the empty field ("Search Community index"). */
    placeholder?: string;
    /** Id of the selected sort option. */
    sort?: string;
    /** The mono line under the field: "1,412 work with api 2 · 189 hidden · showing 41–60". The app builds it. */
    resultLine: string;
    /** Dims and locks the search field and the sort control. */
    disabled?: boolean;
    text: CatalogToolbarText;
    onsearch?: (query: string) => void;
    onsort?: (id: string) => void;
}
