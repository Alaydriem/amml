// BSD-3-Clause. Copyright (c) Charles R. Portwood II. Adapted from the Bedrock Voice Chat client.

// Runtime only. The build-time modules (SvelteSource, Sources) stay out: a barrel is a live import edge, and
// re-exporting them would put the Svelte compiler's parser into the application bundle.
export { CONTEXT_SEPARATOR } from "./Contract.ts";
export { default as I18n } from "./I18n.svelte.ts";
export type { LanguagePack, PackEntry } from "./LanguagePack.ts";
