<script lang="ts">
    import BrandMark from "../primitives/BrandMark.svelte";
    import IconButton from "../primitives/IconButton.svelte";
    import StatusDot from "../primitives/StatusDot.svelte";
    import { Template } from "../../text/Template";
    import StepIndicator from "./StepIndicator.svelte";
    import type { TitleBarProps } from "./TitleBar";

    let { text, steps, registryFetched, registryTone = "ok", onrefresh, onsettings }: TitleBarProps = $props();
</script>

<header class="amml-titlebar">
    <div class="amml-titlebar__brand">
        <BrandMark />
        <span class="amml-titlebar__wordmark">{text.wordmark}</span>
    </div>
    <div class="amml-titlebar__end">
        {#if steps}
            <StepIndicator total={steps.total} current={steps.current} template={text.step} />
        {:else}
            {#if registryFetched !== undefined}
                <span class="amml-titlebar__status">
                    <StatusDot tone={registryTone} caps>{Template.fill(text.registry, { fetched: registryFetched })}</StatusDot>
                </span>
            {/if}
            <IconButton icon="refresh" label={text.refresh} onclick={onrefresh} />
            <IconButton icon="settings" label={text.settings} onclick={onsettings} />
        {/if}
    </div>
</header>

<style>
    .amml-titlebar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: var(--rad-titlebar-height);
        flex-shrink: 0;
        padding-inline: 30px;
        box-sizing: border-box;
        background: var(--color-rad-void);
        border-bottom: 1px solid var(--color-rad-line);
    }

    .amml-titlebar__brand {
        display: flex;
        align-items: center;
        gap: 14px;
    }

    .amml-titlebar__wordmark {
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body);
        line-height: 20px;
        letter-spacing: var(--tracking-rad-brand);
        text-transform: uppercase;
    }

    .amml-titlebar__end {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .amml-titlebar__status {
        padding-inline: 8px;
    }
</style>
