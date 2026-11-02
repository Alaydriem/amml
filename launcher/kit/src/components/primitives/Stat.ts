export interface StatProps {
    /** Mono caps label above the figure. */
    label: string;
    /** The figure, already formatted ("14", "3.2"). */
    value: string;
    /** Unit after the figure ("GB"). */
    unit?: string;
}
