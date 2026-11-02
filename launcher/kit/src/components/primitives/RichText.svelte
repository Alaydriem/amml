<script lang="ts">
    import type { RichTextProps } from "./RichText";

    let { parts }: RichTextProps = $props();
</script>

{#each parts as part, index (index)}{#if part.strong}<strong>{part.text}</strong>{:else}{part.text}{/if}{/each}
