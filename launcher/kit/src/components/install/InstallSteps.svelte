<script lang="ts">
    import InstallStepRow from "./InstallStepRow.svelte";
    import type { InstallStepsProps } from "./InstallSteps";

    let { steps, text, label }: InstallStepsProps = $props();
</script>

<ol class="amml-install-steps" aria-label={label}>
    {#each steps as step (step.id)}
        <li><InstallStepRow {step} {text} /></li>
    {/each}
</ol>

<style>
    .amml-install-steps {
        margin: 0;
        padding: 0;
        list-style: none;
        border-top: 1px solid var(--color-rad-line);
    }
</style>
