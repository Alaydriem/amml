import type { DataMode } from "./DataMode";

/** What the new instance form hands to the app when Create is pressed. */
export interface NewInstanceSubmit {
    name: string;
    version: string;
    dataMode: DataMode;
}
