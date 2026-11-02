<script lang="ts">
    import type { ResultGroup } from "../../model";
    import LoadingRow from "./LoadingRow.svelte";
    import ModResultRow from "./ModResultRow.svelte";
    import { ModResultRowRules } from "./ModResultRow";
    import MoreRow from "./MoreRow.svelte";
    import ResultGroupHead from "./ResultGroupHead.svelte";
    import { ModResultListRules, type ModResultListProps } from "./ModResultList";
    import { PageRequests } from "./PageRequests";
    import type { ResultChunk } from "./ResultChunk";
    import { ResultPaging } from "./ResultPaging";
    import type { SentinelObserver } from "./SentinelObserver";
    import { ViewObserver } from "./ViewObserver";

    let {
        groups,
        checked = [],
        active,
        loading = [],
        generation = 0,
        showGroupCount = false,
        text,
        observerFactory = ViewObserver.create,
        onloadmore,
        onshowmore,
        onactivate,
        onchange,
    }: ModResultListProps = $props();

    const uid = $props.id();
    const requests = new PageRequests();
    // Every mounted sentinel, so a new observer can take them all over.
    const sentinels = new Set<Element>();
    let container: HTMLElement | undefined = $state();
    let observer: SentinelObserver | undefined;
    let anchoredGeneration: number | undefined;
    // The generation the requests belong to; a change drops them.
    let seenGeneration: number | undefined;

    const checkedSet = $derived(new Set(checked));
    const margin = $derived(ResultPaging.lookAhead(groups));
    const busy = $derived(loading.length > 0);

    function enter(element: Element) {
        const ref = ResultPaging.refOf(element);
        if (ref && requests.claim(ref, loading)) onloadmore?.(ref);
    }

    function sentinel(node: Element) {
        sentinels.add(node);
        observer?.observe(node);
        return {
            destroy() {
                sentinels.delete(node);
                observer?.unobserve(node);
            },
        };
    }

    $effect(() => {
        if (!container) return;
        const created = observerFactory(container, enter, margin);
        observer = created;
        for (const node of sentinels) created.observe(node);
        return () => {
            created.disconnect();
            if (observer === created) observer = undefined;
        };
    });

    $effect(() => {
        if (generation === seenGeneration) return;
        seenGeneration = generation;
        requests.reset();
    });

    $effect(() => {
        requests.sync(loading, groups);
    });

    // The first group with earlier rows missing opens at its first loaded row, once for each generation.
    $effect(() => {
        void groups;
        if (!container || anchoredGeneration === generation) return;
        const anchor = container.querySelector<HTMLElement>("[data-anchor]");
        if (!anchor) return;
        anchoredGeneration = generation;
        const head = anchor.closest("[data-group]")?.querySelector<HTMLElement>("[data-head]");
        container.scrollTop = ModResultListRules.anchorTop(anchor.offsetTop, head?.offsetHeight ?? 0);
    });
</script>

<div
    class="amml-results"
    bind:this={container}
    role="region"
    aria-label={text.listLabel}
    aria-busy={busy}
>
    <div class="amml-visually-hidden" role="status" aria-live="polite">{busy ? text.loadingStatus : ""}</div>
    {#each groups as group (group.id)}
        {@const chunks = ResultPaging.chunks(group)}
        {@const before = chunks.filter((chunk) => chunk.side === "before")}
        {@const after = chunks.filter((chunk) => chunk.side === "after")}
        <section class="amml-results__group" data-group={group.id} aria-labelledby="{uid}-{group.id}">
            <ResultGroupHead id="{uid}-{group.id}" heading={ModResultListRules.headLabel(group, showGroupCount, text)} />
            <ul class="amml-results__list">
                {#each before as chunk (ModResultListRules.chunkKey(generation, chunk))}
                    {@render spacer(group, chunk)}
                {/each}
                {#each group.rows as card, index (card.id)}
                    <ModResultRow
                        {card}
                        anchor={index === 0 && before.length > 0}
                        checked={checkedSet.has(card.id)}
                        active={active === card.id}
                        text={{ installed: text.installed, update: text.update, select: text.selectRow }}
                        onchange={(id, next) => onchange?.(ModResultRowRules.toggle(checked, id, next))}
                        {onactivate}
                    />
                {/each}
                {#each after as chunk (ModResultListRules.chunkKey(generation, chunk))}
                    {@render spacer(group, chunk)}
                {/each}
                {#if group.collapsedMore !== undefined}
                    <MoreRow label={ModResultListRules.moreLabel(group, text)} onclick={() => onshowmore?.(group.id)} />
                {/if}
            </ul>
        </section>
    {/each}
</div>

{#snippet spacer(group: ResultGroup, chunk: ResultChunk)}
    <li
        class="amml-results__spacer"
        aria-hidden="true"
        data-registry={group.registryId}
        data-page={chunk.page}
        style:height="{ModResultListRules.spacerHeight(chunk, ResultPaging.ROW_HEIGHT)}px"
        use:sentinel
    >
        <LoadingRow note={ModResultListRules.chunkLabel(chunk, group, text)} />
    </li>
{/snippet}

<style>
    .amml-results {
        position: relative;
        flex: 1 1 0;
        min-height: 0;
        overflow-y: auto;
    }

    .amml-results__list {
        margin: 0;
        padding: 0;
        list-style: none;
    }

    .amml-results__spacer {
        overflow: hidden;
    }
</style>
