import type { PageRef, ResultGroup } from "../../model";
import { Template } from "../../text/Template";
import { Units } from "../../text/Units";
import type { ResultChunk } from "./ResultChunk";
import type { SentinelObserverFactory } from "./SentinelObserverFactory";

export interface ModResultListText {
    /** Accessible name of the list. */
    listLabel: string;
    installed: string;
    update: string;
    /** Template with a `{title}` slot: the checkbox name of a row. */
    selectRow: string;
    /** Template with `{from}` and `{to}` slots, one-based and inclusive within the group: "loading 61–80". */
    loadingRange: string;
    /** Read by a screen reader while a page loads. */
    loadingStatus: string;
    /** Template with `{name}` and `{count}` slots: the group head in search mode, "{name} · {count}". */
    groupCount: string;
    /** Template with `{count}` and `{name}` slots: "+23 more in Community index". */
    more: string;
}

/**
 * Paging contract.
 *
 * - A group's rows sit at registry index positions (`ResultGroup.offset`). The page that holds position `i` is
 *   `floor(i / pageSize) + 1`, and a page is named by `registryId:page`, so groups of one registry share pages.
 * - Every missing run of a group is one spacer per page, `rows * ROW_HEIGHT` px high, with one skeleton row that
 *   carries the "loading from–to" label. The DOM holds one spacer per missing page, not one row per missing row.
 * - A spacer that comes within one page of the visible area calls `onloadmore` once.
 * - The list asks again only after the page is released: its rows arrived, it left `loading` without its rows, or
 *   `generation` changed.
 * - The app bumps `generation` in the same update that replaces the groups (a new query, sort, source or api).
 *   The list then drops every request, re-keys its spacers, and scrolls to the first loaded row once.
 * - The app lists the pages it loads in `loading`, and removes a page when its load ends, with or without rows.
 */
export interface ModResultListProps {
    groups: ResultGroup[];
    /** Ids of the ticked mods. */
    checked?: string[];
    /** Id of the mod shown in the detail pane. */
    active?: string;
    /** Pages the app is loading now. */
    loading?: PageRef[];
    /** Counts the generation of the shown rows; a change drops every request and scrolls to the first loaded row. */
    generation?: number;
    /** Search mode: heads show the group's row total after the name. */
    showGroupCount?: boolean;
    text: ModResultListText;
    /** Replaces the `IntersectionObserver` wrapper. */
    observerFactory?: SentinelObserverFactory;
    onloadmore?: (page: PageRef) => void;
    onshowmore?: (groupId: string) => void;
    onactivate?: (id: string) => void;
    /** The new ticked ids after a row is ticked or cleared. */
    onchange?: (ids: string[]) => void;
}

export class ModResultListRules {
    /** The label of one skeleton run, counted from 1 within its group. */
    static chunkLabel(chunk: ResultChunk, group: ResultGroup, text: ModResultListText): string {
        const start = group.start ?? 0;
        return Template.fill(text.loadingRange, { from: chunk.from - start + 1, to: chunk.to - start });
    }

    /** The group head: the name, with the row total after it in search mode. */
    static headLabel(group: ResultGroup, showCount: boolean, text: ModResultListText): string {
        if (!showCount) return group.label;
        return Template.fill(text.groupCount, { name: group.label, count: Units.en.count(group.total) });
    }

    static moreLabel(group: ResultGroup, text: ModResultListText): string {
        return Template.fill(text.more, { count: Units.en.count(group.collapsedMore ?? 0), name: group.label });
    }

    /** Scroll position that puts the anchor row at the top, below the sticky group head. */
    static anchorTop(anchorOffsetTop: number, headHeight: number): number {
        return Math.max(0, anchorOffsetTop - headHeight);
    }

    /** Key of a spacer: it changes with the generation, so a new generation gets new elements to observe. */
    static chunkKey(generation: number, chunk: ResultChunk): string {
        return `${generation}:${chunk.side}:${chunk.from}`;
    }

    /** Pixel height of the spacer that stands for the rows of a chunk. */
    static spacerHeight(chunk: ResultChunk, rowHeight: number): number {
        return (chunk.to - chunk.from) * rowHeight;
    }
}
