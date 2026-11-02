<script lang="ts">
    import CheckList from "../components/setup/CheckList.svelte";
    import Eyebrow from "../components/primitives/Eyebrow.svelte";
    import WizardFooter from "../components/shell/WizardFooter.svelte";
    import type { FirstRunChecksScreenProps } from "./FirstRunChecksScreen";

    let { checks, text, oninstallcheck, onnext, onskip }: FirstRunChecksScreenProps = $props();
</script>

<div class="amml-screen">
    <div class="amml-screen__body">
        <div class="amml-screen__intro">
            <Eyebrow size="lg">{text.eyebrow}</Eyebrow>
            <h1 class="amml-screen__title">{text.title}</h1>
            <p class="amml-screen__lead">{text.intro}</p>
        </div>
        <div class="amml-screen__panel">
            <CheckList heading={text.checksHeading} {checks} text={text.check} onaction={oninstallcheck} />
        </div>
    </div>
    <WizardFooter
        text={{ note: text.footerNote, secondary: text.skip, primary: text.next }}
        onsecondary={onskip}
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

    .amml-screen__lead {
        margin: 0;
        color: var(--color-rad-dim);
        font-size: var(--text-rad-title-md);
        line-height: 1.68;
    }

    .amml-screen__panel {
        align-self: flex-start;
        flex: 1;
        min-width: 0;
    }
</style>
