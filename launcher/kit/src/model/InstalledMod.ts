import type { ModOrigin } from "./ModOrigin";

/** One mod in an instance (artboard 06). */
export interface InstalledMod {
    /** Mod id, for example `@alaydriem/zoom`. The AMML loader row uses `alaydriem/amml`. */
    id: string;
    title: string;
    origin: ModOrigin;
    /** Name of the registry the mod came from, when `origin` is `registry`. */
    registryName?: string;
    enabled: boolean;
    installedVersion: string;
    /** Newest version that works with this instance; undefined when no registry lists the mod. */
    availableVersion?: string;
    /** Short note after the id, for example "api 2" or "core, always on". */
    note?: string;
}
