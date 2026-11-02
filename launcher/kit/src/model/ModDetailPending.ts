import type { ModCard } from "./ModCard";

/** A mod whose document is still loading: the pane shows what the index row already knows. */
export interface ModDetailPending {
    card: ModCard;
    registryName: string;
    /** Api the shown version is for. */
    api: number;
    /** The file being fetched, as shown in the note (`mods/@carto/mapper.json`). */
    document: string;
}
