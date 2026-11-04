<script lang="ts">
    import Banner from "../primitives/Banner.svelte";
    import Button from "../primitives/Button.svelte";
    import RichText from "../primitives/RichText.svelte";
    import { UpdatesBannerRules, type UpdatesBannerProps } from "./UpdatesBanner";

    let { updates, instances, text, onreview }: UpdatesBannerProps = $props();
</script>

<Banner tone="brand">
    <RichText parts={UpdatesBannerRules.parts(updates, instances, text)} />
    {#snippet action()}
        <Button variant="link" tone="dim" onclick={onreview}>{text.review}</Button>
    {/snippet}
</Banner>
