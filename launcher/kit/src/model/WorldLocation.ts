import type { DataMode } from "./DataMode";
import type { Swatch } from "./Swatch";

/** Where one instance keeps its worlds (artboard 07c). */
export interface WorldLocation {
    instanceId: string;
    name: string;
    swatch: Swatch;
    mode: DataMode;
    /** Shown only for `own`. */
    folder?: string;
}
