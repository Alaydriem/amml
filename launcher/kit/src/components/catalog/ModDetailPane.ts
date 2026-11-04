import type { ModDetail, ModDetailPending } from "../../model";
import { Template } from "../../text/Template";
import { Units } from "../../text/Units";

export interface ModDetailPaneText {
    /** Template with an `{api}` slot: "Version for api {api}". */
    versionFor: string;
    download: string;
    source: string;
    /** Template with `{size}` and `{checksum}` slots: "{size} · {checksum}". */
    downloadValue: string;
    checksum: string;
    noChecksum: string;
    /** Template with `{id}` and `{registry}` slots: "{id} · {registry}". */
    subline: string;
    /** The note under the rows. */
    note: string;
    /** Template with a `{document}` slot, under the rows while the mod document loads: "loading {document}". */
    loading: string;
    /** Shown when no mod is selected. */
    empty: string;
}

/** Show `pending` while the mod document loads, `detail` once it arrived, and the prompt when neither is set. */
export interface ModDetailPaneProps {
    /** The mod to show. */
    detail?: ModDetail;
    /** The mod that is loading; it wins over `detail`. */
    pending?: ModDetailPending;
    text: ModDetailPaneText;
}

export class ModDetailPaneRules {
    /** `0.3 MB · checksum` or `0.3 MB · no checksum`. */
    static downloadValue(detail: ModDetail, text: ModDetailPaneText): string {
        return Template.fill(text.downloadValue, {
            size: Units.en.size(detail.size),
            checksum: detail.hasChecksum ? text.checksum : text.noChecksum,
        });
    }

    static versionLabel(api: number, text: ModDetailPaneText): string {
        return Template.fill(text.versionFor, { api });
    }

    /** `@kv/chunkborders · Community index`. */
    static subline(id: string, registryName: string, text: ModDetailPaneText): string {
        return Template.fill(text.subline, { id, registry: registryName });
    }

    static loadingNote(pending: ModDetailPending, text: ModDetailPaneText): string {
        return Template.fill(text.loading, { document: pending.document });
    }
}
