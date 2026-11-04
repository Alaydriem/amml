/** A registry the launcher reads (artboards 06b, 07a). */
export interface RegistrySource {
    id: string;
    name: string;
    /** Registry URL as shown, without the scheme. */
    url: string;
    builtIn: boolean;
    /** Mods that work with the current api; undefined while unknown. */
    modCount?: number;
    status: "ok" | "offline" | "failed" | "loading";
    /** Free text the app formats, for example "4 min ago". */
    fetched?: string;
}
