import type { ModCard } from "./ModCard";

/** The selected mod in the detail pane: the card plus fields from the mod document. */
export interface ModDetail extends ModCard {
    registryName: string;
    description?: string;
    authors: string[];
    license: string;
    /** Source repository as shown, for example `kv/chunkborders`. */
    sourceLabel: string;
    sourceUrl: string;
    api: number;
    hasChecksum: boolean;
}
