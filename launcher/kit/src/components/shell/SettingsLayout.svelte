<script lang="ts">
    import type { SettingsLayoutProps } from "./SettingsLayout";

    let { nav, children }: SettingsLayoutProps = $props();
</script>

<div class="amml-settings">
    {@render nav()}
    <div class="amml-settings__pane">
        {@render children()}
    </div>
</div>

<style>
    .amml-settings {
        display: flex;
        flex: 1 1 0;
        min-height: 0;
        overflow: clip;
    }

    .amml-settings__pane {
        display: flex;
        flex: 1 1 0;
        flex-direction: column;
        gap: 22px;
        min-width: 0;
        padding: 26px 40px;
        box-sizing: border-box;
        overflow-y: auto;
    }
</style>
