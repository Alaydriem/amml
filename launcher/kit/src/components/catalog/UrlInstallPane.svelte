<script lang="ts">
    import Button from "../primitives/Button.svelte";
    import Callout from "../primitives/Callout.svelte";
    import RichText from "../primitives/RichText.svelte";
    import TextField from "../primitives/TextField.svelte";
    import CheckList from "../setup/CheckList.svelte";
    import { UrlInstallPaneRules, type UrlInstallPaneProps } from "./UrlInstallPane";

    let { url = "", checks, text, unofficial = false, checking = false, oninput, oncheck }: UrlInstallPaneProps = $props();
</script>

<div class="amml-url-pane">
    <p class="amml-url-pane__intro">{text.intro}</p>
    <div class="amml-url-pane__field">
        <div class="amml-url-pane__input">
            <TextField
                value={url}
                mono
                label={text.urlLabel}
                {oninput}
                onsubmit={(value) => {
                    if (UrlInstallPaneRules.canCheck(value, checking)) oncheck?.(value);
                }}
            />
        </div>
        <Button disabled={!UrlInstallPaneRules.canCheck(url, checking)} onclick={() => oncheck?.(url)}>{text.check}</Button>
    </div>
    {#if checks.length > 0}
        <CheckList heading={text.checksLabel} {checks} text={text.checkMarks} />
    {/if}
    {#if unofficial}
        <Callout tone="warn" ground="raised"><RichText parts={UrlInstallPaneRules.unofficial(text)} /></Callout>
    {/if}
</div>

<style>
    .amml-url-pane {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 16px;
        min-width: 0;
        padding: 24px 30px;
        overflow-y: auto;
    }

    .amml-url-pane__intro {
        margin: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-body-sm);
        line-height: 18px;
    }

    .amml-url-pane__field {
        display: flex;
        align-items: stretch;
        gap: 10px;
    }

    .amml-url-pane__input {
        display: flex;
        flex: 1 1 0;
        min-width: 0;
    }

    .amml-url-pane__input :global(.amml-field) {
        flex: 1 1 0;
    }
</style>
