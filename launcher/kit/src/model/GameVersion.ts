import type { Channel } from "./Channel";

/** A game version in the version list (artboards 04, 07b). */
export interface GameVersion {
    version: string;
    channel: Channel;
    state: "installed" | "download" | "unsupported";
    /** Download size in bytes when `state` is `download`; size on disk when `installed`. */
    size?: number;
    /** Instances made from this version. */
    instanceCount: number;
}
