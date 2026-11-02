<script lang="ts">
    import type { AvatarProps } from "./Avatar";

    let { initials, label }: AvatarProps = $props();
</script>

<span class="amml-avatar" role={label ? "img" : undefined} aria-label={label} aria-hidden={label ? undefined : "true"}>
    {initials}
</span>

<style>
    .amml-avatar {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 48px;
        height: 48px;
        flex-shrink: 0;
        background: var(--color-rad-panel);
        border: 1px solid var(--color-rad-line-2);
        border-radius: var(--radius-rad-rail);
        color: var(--color-rad-brand-lift);
        font-size: var(--text-rad-avatar);
        font-weight: var(--font-weight-rad-strong);
        line-height: 22px;
    }
</style>
