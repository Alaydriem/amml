<script lang="ts">
    import AdoptedInstanceRow from "../components/setup/AdoptedInstanceRow.svelte";
    import ChoiceList from "../components/setup/ChoiceList.svelte";
    import Eyebrow from "../components/primitives/Eyebrow.svelte";
    import Panel from "../components/primitives/Panel.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import WizardFooter from "../components/shell/WizardFooter.svelte";
    import { Template } from "../text/Template";
    import type { FirstRunAdoptScreenProps } from "./FirstRunAdoptScreen";

    let { instances, choices, selected, text, onselect, onnext, onback }: FirstRunAdoptScreenProps = $props();
</script>

<div class="amml-screen">
    <div class="amml-screen__body">
        <div class="amml-screen__intro">
            <Eyebrow size="lg">{text.eyebrow}</Eyebrow>
            <h1 class="amml-screen__title"><RichText parts={Template.parts(text.title)} /></h1>
            <p class="amml-screen__lead">{text.intro}</p>
        </div>
        <div class="amml-screen__panels">
            <Panel heading={text.foundHeading} roomy>
                {#each instances as instance (instance.id)}
                    <AdoptedInstanceRow {instance} text={text.adopted} />
                {/each}
            </Panel>
            <ChoiceList heading={text.copyHeading} {choices} value={selected} onchange={onselect} />
        </div>
    </div>
    <WizardFooter
        text={{ note: text.footerNote, secondary: text.back, primary: text.next }}
        onsecondary={onback}
        onprimary={onnext}
    />
</div>

<style>
    .amml-screen {
        display: flex;
        flex: 1;
        flex-direction: column;
        min-height: 0;
    }

    .amml-screen__body {
        display: flex;
        flex: 1;
        gap: 64px;
        min-height: 0;
        padding: 64px 64px 0;
        overflow-y: auto;
    }

    .amml-screen__intro {
        display: flex;
        flex-direction: column;
        flex-shrink: 0;
        gap: 22px;
        width: 440px;
    }

    .amml-screen__title {
        margin: 0;
        color: var(--color-rad-tx);
        font-size: var(--text-rad-hero);
        font-weight: var(--font-weight-rad-light);
        letter-spacing: var(--tracking-rad-display);
        line-height: 1.06;
    }

    .amml-screen__title :global(strong) {
        font-weight: var(--font-weight-rad-strong);
    }

    .amml-screen__lead {
        margin: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-title-md);
        line-height: 1.68;
    }

    .amml-screen__panels {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 18px;
        align-self: flex-start;
        min-width: 0;
    }
</style>
