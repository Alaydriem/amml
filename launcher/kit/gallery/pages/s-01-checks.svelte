<script module lang="ts">
    import type { GalleryMeta } from "../Gallery";
    export const meta: GalleryMeta = { title: "01 First run / Checks", group: "Screens", order: 1, frame: true };
</script>

<script lang="ts">
    import { en } from "../../src/text/en";
    import { AppFrame, TitleBar } from "../../src/components/shell";
    import FirstRunChecksScreen from "../../src/screens/FirstRunChecksScreen.svelte";
    import { SetupFixtures } from "../../src/fixtures/setup";
    import { firstRunChecksEn } from "../../src/text/en/firstRunChecks";

    let checks = $state(SetupFixtures.checks);

    function installCheck(id: string) {
        checks = checks.map((check) =>
            check.id === id
                ? { ...check, severity: "ok" as const, actionLabel: undefined, elevated: undefined, value: "Found" }
                : check,
        );
    }
</script>

<AppFrame>
    {#snippet titlebar()}
        <TitleBar text={en.shell.titleBar} steps={{ total: 3, current: 1 }} />
    {/snippet}
    <FirstRunChecksScreen {checks} text={firstRunChecksEn} oninstallcheck={installCheck} />
</AppFrame>
