<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "Shell", group: "Components", order: 2 };
</script>

<script lang="ts">
    import { Button, Callout, Panel, PanelRow, Toggle } from "../../src/components/primitives";
    import {
        AppFrame,
        BackLink,
        PageHeader,
        SettingsLayout,
        SettingsNav,
        StatusBar,
        TitleBar,
        WizardFooter,
    } from "../../src/components/shell";
    import { FaultBanner } from "../../src/components/dialogs";
    import { DialogFixtures } from "../../src/fixtures/dialogs";
    import { ShellFixtures } from "../../src/fixtures/shell";
    import { en } from "../../src/text/en";

    let active = $state("registries");
    let preview = $state(false);
    let last = $state("");
</script>

<section class="sheet">
    <h3>Title bar</h3>
    <div class="strip">
        <TitleBar
            text={en.shell.titleBar}
            registryFetched={ShellFixtures.registryFetched}
            onrefresh={() => (last = "refresh")}
            onsettings={() => (last = "settings")}
        />
    </div>
    <div class="strip">
        <TitleBar text={en.shell.titleBar} steps={{ total: 3, current: 1 }} />
    </div>
    <p class="muted">Last action: {last}</p>

    <h3>Status bar</h3>
    <div class="strip">
        <StatusBar
            text={en.shell.statusBar}
            version={ShellFixtures.launcherVersion}
            dataPath={ShellFixtures.dataPath}
            storeVersion={ShellFixtures.storeVersion}
        />
    </div>

    <h3>Wizard footer</h3>
    <div class="strip">
        <WizardFooter
            text={{
                note: "The install button may ask for admin permissions to apply and install AMML",
                secondary: "Skip",
                primary: "Next",
            }}
        />
    </div>

    <h3>Page header and back link</h3>
    <div class="pad">
        <BackLink label="Instances" />
        <PageHeader eyebrow="Instances · 3">
            {#snippet title()}Launch <strong>AMML.</strong>{/snippet}
            {#snippet actions()}<Button variant="primary" size="lg" icon="plus">New instance</Button>{/snippet}
        </PageHeader>
        <PageHeader size="section">
            {#snippet title()}Add mods to <strong>1.26.60.05</strong>{/snippet}
        </PageHeader>
    </div>

    <h3>Settings nav</h3>
    <div class="nav-strip">
        <SettingsNav
            text={en.shell.settingsNav}
            items={en.shell.settingsSections}
            value={active}
            onchange={(id) => (active = id)}
        />
    </div>

    <h3>AppFrame, 1280 × 800</h3>
    <div class="frame">
        <AppFrame>
            {#snippet titlebar()}
                <TitleBar text={en.shell.titleBar} registryFetched={ShellFixtures.registryFetched} />
            {/snippet}
            {#snippet banner()}
                <FaultBanner fault={DialogFixtures.faults["operation-failed"]} text={en.faults["operation-failed"]} />
            {/snippet}
            <SettingsLayout>
                {#snippet nav()}
                    <SettingsNav
                        text={en.shell.settingsNav}
                        items={en.shell.settingsSections}
                        value={active}
                        onchange={(id) => (active = id)}
                    />
                {/snippet}
                <Panel heading="Builds">
                    <PanelRow>
                        {#snippet title()}Show preview builds{/snippet}
                        {#snippet detail()}Needs Minecraft Preview from the Store. Few mods support them.{/snippet}
                        {#snippet aside()}<Toggle checked={preview} label="Show preview builds" onchange={(next) => (preview = next)} />{/snippet}
                    </PanelRow>
                    <PanelRow>
                        {#snippet title()}Version list{/snippet}
                        {#snippet detail()}
                            <code>MinecraftBedrockArchiver/GdkLinks · urls.json · 212 release, 31 preview</code>
                        {/snippet}
                        {#snippet aside()}<span class="mono">4 min ago</span>{/snippet}
                    </PanelRow>
                </Panel>
                <Callout tone="warn" ground="raised">
                    <strong>An installed version is the clean copy every instance of it starts from.</strong>
                </Callout>
            </SettingsLayout>
            {#snippet statusbar()}
                <StatusBar
                    text={en.shell.statusBar}
                    version={ShellFixtures.launcherVersion}
                    dataPath={ShellFixtures.dataPath}
                    storeVersion={ShellFixtures.storeVersion}
                />
            {/snippet}
        </AppFrame>
    </div>
</section>

<style>
    .sheet {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    h3 {
        margin: 12px 0 0;
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
        font-weight: var(--font-weight-rad-regular);
        letter-spacing: var(--tracking-rad-label);
        text-transform: uppercase;
    }

    .strip {
        width: 1280px;
        max-width: 100%;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line);
    }

    .pad {
        display: flex;
        flex-direction: column;
        gap: 18px;
        width: 1200px;
        max-width: 100%;
        padding: 22px 40px;
        box-sizing: border-box;
        background: var(--color-rad-void);
        border: 1px solid var(--color-rad-line);
    }

    .nav-strip {
        display: flex;
        height: 380px;
        width: 240px;
    }

    .frame {
        width: 1280px;
        height: 800px;
        border: 1px solid var(--color-rad-line);
    }

    .muted,
    .mono,
    code {
        color: var(--color-rad-faint);
        font-family: var(--font-rad-mono);
        font-size: var(--text-rad-mono);
    }

    p.muted {
        margin: 0;
    }
</style>
