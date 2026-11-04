/**
 * AMML launcher kit: the Svelte 5 components and screens of the Paper page "Launcher v1".
 *
 * Import `css/tokens.css` and `css/base.css` once in the app. Screens are controlled: pass model data and text,
 * handle the `on…` callbacks. Nothing here calls Tauri, the network or storage. Fixtures are a separate entry
 * (`@amml/launcher-kit/fixtures`).
 */
export type * from "./model";
export * from "./text";
export { en, type LauncherText } from "./text/en";

export * from "./components/primitives";
export * from "./components/shell";
export * from "./components/setup";
export * from "./components/instances";
export * from "./components/install";
export * from "./components/mods";
export * from "./components/catalog";
export * from "./components/settings";
export * from "./components/dialogs";
export * from "./screens";
