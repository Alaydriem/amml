<script lang="ts">
    import { Template } from "../../text/Template";
    import Avatar from "../primitives/Avatar.svelte";
    import Button from "../primitives/Button.svelte";
    import type { AuthorCardProps } from "./AuthorCard";

    let { author, text, onopenlink }: AuthorCardProps = $props();
</script>

<div class="amml-author">
    <Avatar initials={author.initials} />
    <div class="amml-author__body">
        <div class="amml-author__byline">{Template.fill(text.byline, { name: author.name })}</div>
        <div class="amml-author__link">{author.link}</div>
    </div>
    <Button variant="secondary" icon="github" onclick={() => onopenlink?.(author.url)}>{text.open}</Button>
</div>

<style>
    .amml-author {
        display: flex;
        align-items: center;
        gap: 18px;
        padding: 16px 18px;
        background: var(--color-rad-void-2);
        border: 1px solid var(--color-rad-line);
        border-radius: var(--radius-rad-lg);
    }

    .amml-author__body {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 3px;
        min-width: 0;
    }

    .amml-author__byline {
        color: var(--color-rad-tx);
        font-size: var(--text-rad-lead);
        line-height: 22px;
    }

    .amml-author__link {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        line-height: 16px;
    }
</style>
