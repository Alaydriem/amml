export interface LinkRowProps {
    /** The row's name ("Source code"). */
    label: string;
    /** Mono link text ("Alaydriem/amml"). */
    value: string;
    /** Address passed to `onopenlink`. */
    href: string;
    onopenlink?: (href: string) => void;
}
