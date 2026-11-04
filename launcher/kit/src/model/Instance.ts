import type { DataMode } from "./DataMode";
import type { Swatch } from "./Swatch";

/** One instance folder, as the instance list and the instance detail show it. */
export interface Instance {
    id: string;
    name: string;
    swatch: Swatch;
    gameVersion: string;
    ammlVersion: string;
    modCount: number;
    dataMode: DataMode;
    /** Folder path as shown to the user, for example `%LOCALAPPDATA%\amml\instances\1.26.60.05`. */
    folder: string;
    /** Free text the app formats, for example "last played 2 h ago" or "running since 14:02". */
    activity: string;
    running: boolean;
    updateCount: number;
}
