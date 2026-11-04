<script lang="ts">
    import { GalleryIndex } from "./Gallery";

    const pages = GalleryIndex.load();
    const groups = [...new Set(pages.map((p) => p.meta.group))];

    let slug = $state(location.hash.slice(1) || pages[0]?.slug || "");
    const current = $derived(pages.find((p) => p.slug === slug) ?? pages[0]);

    $effect(() => {
        const onhash = () => (slug = location.hash.slice(1));
        addEventListener("hashchange", onhash);
        return () => removeEventListener("hashchange", onhash);
    });
</script>

<div class="g">
    <nav class="g__nav">
        <p class="g__brand">AMML launcher kit</p>
        {#each groups as group (group)}
            <p class="g__group">{group}</p>
            {#each pages.filter((p) => p.meta.group === group) as page (page.slug)}
                <a class="g__link" class:g__link--on={page.slug === current?.slug} href="#{page.slug}">{page.meta.title}</a>
            {/each}
        {/each}
    </nav>
    <main class="g__main">
        {#if current}
            {#if current.meta.frame}
                <div class="g__frame"><current.component /></div>
            {:else}
                <div class="g__sheet"><current.component /></div>
            {/if}
        {/if}
    </main>
</div>

<style>
    .g {
        display: flex;
        min-height: 100vh;
        background: #120b22;
    }

    .g__nav {
        width: 240px;
        flex-shrink: 0;
        padding: 24px 16px;
        border-right: 1px solid var(--color-rad-line);
        position: sticky;
        top: 0;
        height: 100vh;
        overflow-y: auto;
    }

    .g__brand {
        margin: 0 0 16px;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
        color: var(--color-rad-dim);
    }

    .g__group {
        margin: 16px 0 6px;
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
        color: var(--color-rad-faint);
    }

    .g__link {
        display: block;
        padding: 5px 10px;
        border-radius: var(--radius-rad-sm);
        color: var(--color-rad-dim);
        text-decoration: none;
        font-size: var(--text-rad-small);
    }

    .g__link--on {
        background: var(--color-rad-panel);
        color: var(--color-rad-tx);
    }

    .g__main {
        flex: 1;
        padding: 32px;
        overflow: auto;
    }

    .g__frame {
        width: 1280px;
        height: 800px;
        border-radius: var(--radius-rad-lg);
        overflow: hidden;
        box-shadow: 0 0 0 1px var(--color-rad-line);
    }

    .g__sheet {
        max-width: 1280px;
    }
</style>
