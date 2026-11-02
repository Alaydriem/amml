import type { Swatch } from "./Swatch";

/** An instance folder the launcher found on this PC and can take over (artboard 02). */
export interface AdoptedInstance {
    id: string;
    name: string;
    swatch: Swatch;
    /** AMML version found in the folder; undefined when it cannot be read. */
    ammlVersion?: string;
    modCount: number;
}
