<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = {
        title: "02 First run / Adopt and data",
        group: "Screens",
        order: 2,
        frame: true,
    };
</script>

<script lang="ts">
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import FirstRunAdoptScreen from "../../src/screens/FirstRunAdoptScreen.svelte";
    import { SetupFixtures } from "../../src/fixtures/setup";
    import { firstRunAdoptEn } from "../../src/text/en/firstRunAdopt";

    let selected = $state<string | undefined>(SetupFixtures.selectedChoice);
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar text={en.shell.titleBar} steps={{ total: 3, current: 2 }} />
    {/snippet}
    <FirstRunAdoptScreen
        instances={SetupFixtures.adopted}
        choices={SetupFixtures.choices}
        {selected}
        onselect={(id) => (selected = id)}
        text={firstRunAdoptEn}
    />
</AppFrame>
