import type { Plural } from "../../text/Template";

export interface InstalledVersionRowText {
    /** `{count}` instances use the version. */
    instances: Plural;
    noInstances: string;
    remove: string;
}

export interface InstalledVersionRowProps {
    version: string;
    /** Instances made from this version. Remove is enabled only at zero. */
    instanceCount: number;
    /** Size on disk as shown, already formatted. */
    sizeText: string;
    text: InstalledVersionRowText;
    onremove?: (version: string) => void;
}

export class InstalledVersionRowRules {
    /** A version in use by an instance cannot be removed. */
    static canRemove(instanceCount: number): boolean {
        return instanceCount === 0;
    }
}
