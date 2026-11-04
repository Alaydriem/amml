import type { CardView } from "#lib/bindings/CardView.ts";
import type { CatalogLayoutView } from "#lib/bindings/CatalogLayoutView.ts";
import type { ModDetailView } from "#lib/bindings/ModDetailView.ts";
import type { ModCard, ModDetail, ResultGroup } from "@amml/launcher-kit";
import { SwatchRules } from "./SwatchRules.ts";

// Turns catalogue facts into the kit's card, detail and group models.
export class CatalogPresenter {
    static toCard(view: CardView): ModCard {
        return {
            id: view.id,
            registryId: view.registryId,
            title: view.title,
            summary: view.summary,
            tags: view.tags,
            version: view.version,
            size: view.size,
            icon: view.iconUrl ?? undefined,
            swatch: SwatchRules.forName(view.id),
            state: view.state,
        };
    }

    static toDetail(view: ModDetailView, registryName: string): ModDetail {
        return {
            ...CatalogPresenter.toCard(view.card),
            registryName,
            description: view.description ?? undefined,
            authors: view.authors,
            license: view.license,
            sourceLabel: view.source.replace(/^https?:\/\/(www\.)?github\.com\//, ""),
            sourceUrl: view.source,
            api: view.api,
            hasChecksum: view.sha256 !== "",
        };
    }

    // The groups of a browse list. `loaded` holds the cards of each page that has come, by page number (from 1).
    // A group's rows start at the first loaded page it touches and run on while the next pages are loaded too.
    static toGroups(layout: CatalogLayoutView, loaded: ReadonlyMap<number, readonly ModCard[]>): ResultGroup[] {
        return layout.groups.map((group) => {
            const first = Math.floor(group.start / layout.pageSize) + 1;
            const last = Math.floor((group.start + group.total - 1) / layout.pageSize) + 1;
            let from = first;
            while (from <= last && !loaded.has(from)) from += 1;
            const base = (from - 1) * layout.pageSize;
            const run: ModCard[] = [];
            for (let page = from; page <= last && loaded.has(page); page += 1) run.push(...(loaded.get(page) ?? []));
            const low = Math.max(group.start, base);
            const high = Math.min(group.start + group.total, base + run.length);
            return {
                id: group.id,
                label: group.label,
                registryId: layout.registryId,
                rows: from > last ? [] : run.slice(low - base, Math.max(low - base, high - base)),
                offset: from > last ? group.start : low,
                start: group.start,
                total: group.total,
                pageSize: layout.pageSize,
            };
        });
    }
}
