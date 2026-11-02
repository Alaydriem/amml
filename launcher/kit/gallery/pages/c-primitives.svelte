<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "Primitives", group: "Components", order: 1 };
</script>

<script lang="ts">
    import {
        Avatar,
        Banner,
        BrandMark,
        Button,
        Callout,
        Checkbox,
        ContextChip,
        Dialog,
        EmptyState,
        Eyebrow,
        ICONS,
        Icon,
        IconButton,
        KeyValueRow,
        Menu,
        Note,
        Panel,
        PanelRow,
        Pill,
        ProgressBar,
        RichText,
        SearchField,
        SectionLabel,
        Segmented,
        Select,
        Skeleton,
        Stat,
        StatusDot,
        StatusMark,
        Tag,
        TextField,
        Tile,
        Toggle,
        type IconName,
    } from "../../src/components/primitives";
    import { en } from "../../src/text/en";
    import { Template } from "../../src/text/Template";

    const iconNames = Object.keys(ICONS) as IconName[];

    let toggleOn = $state(true);
    let toggleOff = $state(false);
    let ticked = $state(true);
    let unticked = $state(false);
    let marker = $state(true);
    let channel = $state("release");
    let worlds = $state("shared");
    let name = $state("1.26.60.05-players");
    let url = $state("https://example.org/nightvision-1.2.0.zip");
    let query = $state("");
    let sort = $state("name");
    let limit = $state("10");
    let picked = $state("none");
    let dialogOpen = $state(false);
</script>

<section class="sheet">
    <h3>Buttons</h3>
    <div class="row">
        <Button variant="primary" size="lg" icon="plus">New instance</Button>
        <Button>Open folder</Button>
        <Button variant="ghost">Show log</Button>
        <Button variant="danger">Delete folder</Button>
        <Button variant="warn">Install unofficial</Button>
        <Button variant="launch" icon="play">Launch</Button>
        <Button variant="running">Running</Button>
        <Button variant="primary" caps size="lg">Next</Button>
        <Button variant="ghost" caps size="lg">Skip</Button>
        <Button variant="primary" caps>Create</Button>
        <Button variant="ghost" caps>Cancel</Button>
        <Button variant="warn" caps>Install unofficial</Button>
        <Button variant="primary" caps size="sm">Install 2 mods</Button>
        <Button disabled>Disabled</Button>
        <Button variant="link">Add a registry URL</Button>
        <Button variant="link" tone="fault">Remove</Button>
        <Button variant="link" tone="dim">Copy all</Button>
    </div>

    <h3>Icons</h3>
    <div class="row">
        {#each iconNames as iconName (iconName)}
            <span class="icon-cell"><Icon name={iconName} size={20} /><code>{iconName}</code></span>
        {/each}
    </div>
    <div class="row">
        <IconButton icon="refresh" label="Refresh" />
        <IconButton icon="settings" label="Settings" />
        <IconButton icon="refresh" label="Disabled" disabled />
        <BrandMark />
        <BrandMark size="lg" />
    </div>

    <h3>Toggle, checkbox, segmented</h3>
    <div class="row">
        <Toggle checked={toggleOff} label="Show preview builds" onchange={(next) => (toggleOff = next)} />
        <Toggle checked={toggleOn} label="Enabled" onchange={(next) => (toggleOn = next)} />
        <Toggle label="Disabled" disabled />
        <Toggle size="sm" label="Show mods that need another AMML" />
        <Toggle size="sm" checked label="Small on" />
    </div>
    <div class="row">
        <Checkbox checked={unticked} label="Not selected" onchange={(next) => (unticked = next)} />
        <Checkbox checked={ticked} label="Selected" onchange={(next) => (ticked = next)} />
        <Checkbox label="Disabled" disabled />
        <Checkbox variant="marker" checked={marker} label="Version" onchange={(next) => (marker = next)} />
        <Checkbox variant="marker" label="Version" />
        <Checkbox decorative variant="marker" checked />
        <Checkbox decorative variant="marker" />
    </div>
    <div class="row">
        <Segmented
            label="Channel"
            options={[
                { id: "release", label: "Release" },
                { id: "preview", label: "Preview" },
            ]}
            value={channel}
            onchange={(id) => (channel = id)}
        />
        <Segmented
            label="Worlds and packs"
            options={[
                { id: "shared", label: "Shared" },
                { id: "own", label: "Own folder" },
            ]}
            value={worlds}
            onchange={(id) => (worlds = id)}
        />
    </div>

    <h3>Fields</h3>
    <div class="column">
        <Eyebrow size="md">Instance name</Eyebrow>
        <TextField value={name} label="Instance name" mono oninput={(next) => (name = next)} />
        <TextField value={url} label="Registry id or zip URL" mono oninput={(next) => (url = next)} />
        <TextField label="Empty" placeholder="Type here" />
        <SearchField value={query} label="Search" placeholder="Search Community index" oninput={(next) => (query = next)} />
        <div class="row">
            <Select
                label="Sort"
                compact
                options={[
                    { id: "name", label: "Name" },
                    { id: "updated", label: "Updated" },
                ]}
                value={sort}
                onchange={(id) => (sort = id)}
            />
            <Select
                label="Size limit"
                options={[
                    { id: "5", label: "5 GB" },
                    { id: "10", label: "10 GB" },
                    { id: "20", label: "20 GB" },
                ]}
                value={limit}
                onchange={(id) => (limit = id)}
            />
        </div>
    </div>

    <h3>Pills, tags, status</h3>
    <div class="row">
        <Pill tone="brand" dot tint>2 updates</Pill>
        <Pill tone="ok" dot>Installed</Pill>
        <Pill tone="brand" dot>~4.1 GB download</Pill>
        <Pill tone="muted" dot>Not supported</Pill>
        <Pill tone="warn">Unofficial</Pill>
        <Pill tone="fault">Failed</Pill>
        <Pill tone="brand" size="sm" onclick={() => (picked = "update")}>Update</Pill>
        <Tag>Core</Tag>
        <Tag bordered={false}>Built in</Tag>
        <Tag tone="ok" bordered={false}>Kept</Tag>
        <StatusDot tone="ok" size="sm">Up to date</StatusDot>
        <StatusDot tone="ok">Store Bedrock 1.26.60.5 registered</StatusDot>
        <StatusDot tone="ok" caps>Registry · 4 min ago</StatusDot>
        <StatusDot tone="warn" caps>Registry · stale</StatusDot>
        <StatusMark state="ok" />
        <StatusMark state="warn" />
        <StatusMark state="active" />
        <StatusMark state="pending" />
        <StatusMark state="ok" size={20} />
        <StatusMark state="failed" label="Failed" />
    </div>
    <div class="row">
        <ContextChip
            text={en.shell.contextChip}
            parts={[
                { id: "game", label: "Game", value: "1.26.60.5" },
                { id: "amml", label: "AMML", value: "26.60.5" },
                { id: "api", label: "api", value: "2", tone: "ok" },
            ]}
        />
    </div>

    <h3>Tiles, avatar, progress, skeleton</h3>
    <div class="row">
        <Tile size="header" swatch={3} />
        <Tile size="rail" swatch={5} />
        <Tile size="detail" swatch={6} />
        <Tile size="row" swatch={4} />
        <Tile size="rail" icon="shield" label="Shield" />
        <Tile size="dot" swatch={7} />
        <Avatar initials="CP" />
    </div>
    <div class="column narrow">
        <ProgressBar value={33} label="Disk use" />
        <ProgressBar value={62} label="Download" />
        <div class="skeleton-row">
            <Skeleton width="28px" height={28} tone="soft" radius="seg" />
            <div class="skeleton-lines">
                <Skeleton width="38%" height={12} />
                <Skeleton width="64%" height={10} tone="soft" />
            </div>
        </div>
    </div>

    <h3>Callouts and banners</h3>
    <div class="column">
        <Callout>Installs <strong>AMML 26.60.5</strong> and <strong>6 default mods</strong>. About 1.9 GB on disk.</Callout>
        <Callout tone="warn">
            <strong>AMML is creating your new instance. Do not start Minecraft from the Store during this install.</strong>
            This may take a minute or two to complete.
        </Callout>
        <Callout tone="warn" ground="base">
            <strong>Not in any registry.</strong> It installs as unofficial: no checksum, and the launcher never checks it
            for updates.
        </Callout>
        <Banner tone="brand">
            <strong>3 updates available</strong> in 2 instances. Nothing is installed until you ask.
            {#snippet action()}<Button variant="ghost" size="sm">Review</Button>{/snippet}
        </Banner>
        <Banner tone="fault">
            <strong>Minecraft is not installed from the Store.</strong> The launcher needs your Store copy to install
            versions.
            {#snippet action()}<Button size="sm">Open the Store</Button>{/snippet}
        </Banner>
        <Banner tone="warn">
            <strong>Not enough space on C:\.</strong> This install needs <code>6.1 GB</code> at once.
            {#snippet action()}<Button size="sm">Check again</Button>{/snippet}
        </Banner>
        <Banner tone="fault">
            <strong>Download failed on both mirrors.</strong> 1.28 of 2.06 GB is kept.
            {#snippet action()}<Button variant="primary" size="sm">Try again</Button>{/snippet}
        </Banner>
        <Banner tone="info">
            <strong>The last install was cut off.</strong> The launcher finished putting your Store version back.
            {#snippet action()}<Button size="sm">Show log</Button>{/snippet}
        </Banner>
    </div>

    <h3>Panel, rows, stats</h3>
    <div class="column">
        <Panel heading="Registries">
            {#snippet aside()}<span class="link">Add a registry URL</span>{/snippet}
            <PanelRow>
                {#snippet title()}Official{/snippet}
                {#snippet detail()}<code>alaydriem.github.io/amml-registry/registry.json</code>{/snippet}
                {#snippet aside()}<code class="ok">12 mods · 4 min ago</code>{/snippet}
            </PanelRow>
            <PanelRow>
                {#snippet leading()}<StatusMark state="ok" />{/snippet}
                {#snippet title()}Gaming Services{/snippet}
                {#snippet detail()}Needed to install game versions{/snippet}
                {#snippet aside()}<Toggle label="Gaming Services" checked />{/snippet}
            </PanelRow>
        </Panel>
        <Panel heading="Shared folder">
            <PanelRow size="sm">
                {#snippet title()}Source code{/snippet}
                {#snippet aside()}<code class="link">Alaydriem/amml</code>{/snippet}
            </PanelRow>
            <div class="stats">
                <Stat label="Worlds" value="14" />
                <Stat label="Resource packs" value="6" />
                <Stat label="Behavior packs" value="3" />
                <Stat label="On disk" value="3.2" unit="GB" />
            </div>
        </Panel>
        <div class="detail">
            <KeyValueRow label="Version for api 2" value="1.1.2" />
            <KeyValueRow label="Download" value="0.3 MB · checksum" tone="dim" />
            <KeyValueRow label="Source" value="kv/chunkborders" tone="brand" />
        </div>
        <SectionLabel>C</SectionLabel>
        <Note>Instances keep their own mods. <strong>Shared data</strong> lives in <code>%LOCALAPPDATA%\amml\data</code>.</Note>
        <p class="muted"><RichText parts={Template.parts("**{count} updates available** in {where}", { count: 3, where: "2 instances" })} /></p>
    </div>

    <h3>Empty state</h3>
    <div class="column narrow-empty">
        <EmptyState title="No instances yet.">
            {#snippet body()}An instance is one game version with AMML and its mods, in a folder of its own.{/snippet}
            {#snippet actions()}<Button variant="primary" icon="plus">New instance</Button>{/snippet}
        </EmptyState>
        <EmptyState icon="alert-circle" title="Old test registry is offline.">
            {#snippet body()}The launcher could not reach the registry.{/snippet}
            {#snippet actions()}<Button>Try again</Button><Button variant="ghost">Remove it</Button>{/snippet}
        </EmptyState>
    </div>

    <h3>Menu and dialog</h3>
    <div class="row">
        <Menu
            label="More actions"
            items={[
                { id: "open", label: "Open folder" },
                { id: "rename", label: "Rename" },
                { id: "delete", label: "Delete", tone: "fault", separated: true },
            ]}
            onselect={(id) => (picked = id)}
        />
        <span class="muted">Chosen: {picked}</span>
        <Button onclick={() => (dialogOpen = true)}>Open dialog</Button>
        <Dialog open={dialogOpen} title="Delete 1.26.52.3-prores?" onclose={() => (dialogOpen = false)}>
            This deletes the folder below, with its AMML, mods and mod settings.
            {#snippet actions()}
                <Button variant="ghost" onclick={() => (dialogOpen = false)}>Keep it</Button>
                <Button variant="danger" onclick={() => (dialogOpen = false)}>Delete folder</Button>
            {/snippet}
        </Dialog>
    </div>
    <div class="row top">
        <Dialog inline eyebrow="Needs your OK" title="Turn on Developer Mode?">
            Windows would not let the launcher copy <strong>Minecraft.Windows.exe</strong> for 1.26.51.1. With Developer Mode
            on, it can. Windows asks for admin rights once.
            {#snippet actions()}
                <Button variant="ghost">Stop install</Button>
                <Button variant="primary">Turn it on</Button>
            {/snippet}
        </Dialog>
        <Dialog inline size="lg" title="Add a mod to 1.26.60.05">
            <TextField label="Registry id or zip URL" value="https://example.org/nightvision-1.2.0.zip" mono />
            <Callout tone="warn" ground="base"><strong>Not in any registry.</strong> It installs as unofficial.</Callout>
            {#snippet actions()}
                <Button variant="ghost">Cancel</Button>
                <Button variant="warn">Install unofficial</Button>
            {/snippet}
        </Dialog>
    </div>
</section>

<style>
    .sheet {
        display: flex;
        flex-direction: column;
        gap: 20px;
        max-width: 900px;
    }

    h3 {
        margin: 12px 0 0;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        font-weight: 400;
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
    }

    .row {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        align-items: center;
    }

    .row.top {
        align-items: flex-start;
    }

    .column {
        display: flex;
        flex-direction: column;
        gap: 12px;
    }

    .narrow {
        max-width: 420px;
    }

    .narrow-empty {
        max-width: 620px;
    }

    .icon-cell {
        display: inline-flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
        min-width: 84px;
        color: var(--color-rad-dim);
    }

    .icon-cell code,
    .muted {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-label);
    }

    .skeleton-row {
        display: flex;
        gap: 14px;
        align-items: center;
    }

    .skeleton-lines {
        display: flex;
        flex: 1;
        flex-direction: column;
        gap: 7px;
    }

    .stats {
        display: flex;
        padding: 14px 18px;
    }

    .detail {
        max-width: 340px;
    }

    .link {
        color: var(--color-rad-brand-lift);
    }

    code {
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
    }

    code.ok {
        color: var(--color-rad-ok);
    }
</style>
