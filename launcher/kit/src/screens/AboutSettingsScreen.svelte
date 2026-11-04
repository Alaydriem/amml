<script lang="ts">
    import Panel from "../components/primitives/Panel.svelte";
    import AboutHeader from "../components/settings/AboutHeader.svelte";
    import AuthorCard from "../components/settings/AuthorCard.svelte";
    import BugReportPanel from "../components/settings/BugReportPanel.svelte";
    import LinkRow from "../components/settings/LinkRow.svelte";
    import { AboutSettingsScreenRules, type AboutSettingsScreenProps } from "./AboutSettingsScreen";

    let {
        version,
        builtOn,
        platform,
        updateVersion,
        author,
        links,
        report,
        text,
        onupdate,
        onopenlink,
        oncopyreport,
    }: AboutSettingsScreenProps = $props();
</script>

<AboutHeader
    text={text.header}
    build={AboutSettingsScreenRules.build({ version, builtOn, platform }, text)}
    {updateVersion}
    {onupdate}
/>

<AuthorCard {author} text={text.author} {onopenlink} />

<div class="amml-about__cards">
    <div class="amml-about__card">
        <Panel heading={text.projectHeading}>
            {#each links as link (link.id)}
                <LinkRow label={text.links[link.id].label} value={text.links[link.id].value} href={link.href} {onopenlink} />
            {/each}
        </Panel>
    </div>
    <div class="amml-about__card">
        <BugReportPanel text={text.bugReport} lines={report} oncopy={oncopyreport} {onopenlink} />
    </div>
</div>

<p class="amml-about__disclaimer">{text.disclaimer}</p>

<style>
    .amml-about__cards {
        display: flex;
        align-items: flex-start;
        gap: 18px;
    }

    .amml-about__card {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        min-width: 0;
    }

    .amml-about__disclaimer {
        margin: 0;
        color: var(--color-rad-faint);
        font-size: var(--text-rad-small);
        line-height: 1.6;
    }
</style>
