<script lang="ts">
    import InstallSteps from "../components/install/InstallSteps.svelte";
    import ProgressReadout from "../components/install/ProgressReadout.svelte";
    import FaultBanner from "../components/dialogs/FaultBanner.svelte";
    import Button from "../components/primitives/Button.svelte";
    import Callout from "../components/primitives/Callout.svelte";
    import Eyebrow from "../components/primitives/Eyebrow.svelte";
    import RichText from "../components/primitives/RichText.svelte";
    import BackLink from "../components/shell/BackLink.svelte";
    import PageHeader from "../components/shell/PageHeader.svelte";
    import { Template } from "../text/Template";
    import type { InstallProgressScreenProps } from "./InstallProgressScreen";

    let {
        name,
        steps,
        progress,
        stepNumber,
        stepTotal,
        text,
        fault,
        faultText,
        restored = false,
        onback,
        oncancel,
        onretry,
        onshowlog,
    }: InstallProgressScreenProps = $props();

    const failed = $derived(fault !== undefined);
    const position = $derived({ current: stepNumber, total: stepTotal });
</script>

<div class="amml-screen">
    <div class="amml-screen__back">
        <BackLink label={text.back} onclick={onback} />
    </div>
    <div class="amml-screen__title">
        <PageHeader>
            {#snippet title()}
                <RichText parts={Template.parts(failed ? text.failedTitle : text.title, { name })} />
            {/snippet}
        </PageHeader>
    </div>
    <div class="amml-screen__columns">
        <div class="amml-screen__steps">
            <InstallSteps {steps} text={text.steps} label={text.stepsLabel} />
        </div>
        <div class="amml-screen__status">
            {#if fault && faultText}
                <Eyebrow size="lg">{Template.fill(text.stopped, position)}</Eyebrow>
                <FaultBanner {fault} text={faultText} />
                {#if restored}
                    <Callout tone="ok">
                        <RichText parts={Template.parts(text.restored)} />
                    </Callout>
                {/if}
                <div class="amml-screen__actions">
                    <Button variant="primary" onclick={onretry}>{text.retry}</Button>
                    <Button variant="ghost" onclick={onshowlog}>{text.showLog}</Button>
                </div>
            {:else}
                {#if progress}
                    <ProgressReadout eyebrow={Template.fill(text.step, position)} {progress} text={text.readout} />
                {/if}
                <Callout tone="warn" ground="raised">
                    <RichText parts={Template.parts(text.warning)} />
                </Callout>
                <div class="amml-screen__actions">
                    <Button variant="secondary" onclick={oncancel}>{text.cancel}</Button>
                    <Button variant="ghost" onclick={onshowlog}>{text.showLog}</Button>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .amml-screen {
        display: flex;
        flex: 1;
        flex-direction: column;
        min-height: 0;
        padding: 22px 40px 0;
    }

    .amml-screen__back {
        padding-bottom: 14px;
    }

    .amml-screen__title {
        padding-bottom: 22px;
    }

    .amml-screen__columns {
        display: flex;
        flex: 1;
        gap: 40px;
        min-height: 0;
    }

    .amml-screen__steps {
        flex-shrink: 0;
        width: 470px;
    }

    .amml-screen__status {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 18px;
        min-width: 0;
    }

    .amml-screen__actions {
        display: flex;
        gap: 10px;
    }
</style>
