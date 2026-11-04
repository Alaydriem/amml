import type { PageRef, ResultGroup } from "../../model";
import type { ResultChunk } from "./ResultChunk";

/** Which index pages of a group are missing, and the sizes the list draws them at. */
export class ResultPaging {
    /** Height in px of one row and of one skeleton row, including its divider. A spacer is `rows * ROW_HEIGHT`. */
    static readonly ROW_HEIGHT = 57;

    /** The registry index page, counted from 1, that holds index position `index`. */
    static page(index: number, pageSize: number): number {
        return Math.floor(index / pageSize) + 1;
    }

    static key(ref: PageRef): string {
        return `${ref.registryId}:${ref.page}`;
    }

    /**
     * The runs of rows a group lacks, in display order: every unloaded row before the first loaded row, then every
     * unloaded row after the last. Each run stays inside one registry index page. A group that shows a "more" row
     * never pages.
     */
    static chunks(group: ResultGroup): ResultChunk[] {
        if (group.collapsedMore !== undefined || group.pageSize <= 0 || group.total <= 0) return [];
        const start = group.start ?? 0;
        const end = start + group.total;
        const loadedStart = Math.min(Math.max(group.offset, start), end);
        const loadedEnd = Math.min(Math.max(group.offset + group.rows.length, loadedStart), end);
        return [
            ...ResultPaging.split("before", start, loadedStart, group.pageSize),
            ...ResultPaging.split("after", loadedEnd, end, group.pageSize),
        ];
    }

    /** Registry index pages with at least one unloaded row of the group, in order, each once. */
    static missingPages(group: ResultGroup): number[] {
        const pages: number[] = [];
        for (const chunk of ResultPaging.chunks(group)) {
            if (!pages.includes(chunk.page)) pages.push(chunk.page);
        }
        return pages;
    }

    /** The keys of every page that any group still lacks. */
    static missingKeys(groups: readonly ResultGroup[]): Set<string> {
        const keys = new Set<string>();
        for (const group of groups) {
            for (const page of ResultPaging.missingPages(group)) {
                keys.add(ResultPaging.key({ registryId: group.registryId, page }));
            }
        }
        return keys;
    }

    /** How far the observer looks ahead, in px: the tallest page of any group. */
    static lookAhead(groups: readonly ResultGroup[]): number {
        return groups.reduce((tallest, group) => Math.max(tallest, group.pageSize), 0) * ResultPaging.ROW_HEIGHT;
    }

    /** Reads the page a sentinel element stands for. */
    static refOf(element: Element): PageRef | undefined {
        const registryId = element.getAttribute("data-registry");
        const page = Number(element.getAttribute("data-page"));
        if (registryId === null || !Number.isInteger(page)) return undefined;
        return { registryId, page };
    }

    private static split(side: ResultChunk["side"], from: number, to: number, pageSize: number): ResultChunk[] {
        const chunks: ResultChunk[] = [];
        let cursor = from;
        while (cursor < to) {
            const page = ResultPaging.page(cursor, pageSize);
            const stop = Math.min(page * pageSize, to);
            chunks.push({ side, page, from: cursor, to: stop });
            cursor = stop;
        }
        return chunks;
    }
}
