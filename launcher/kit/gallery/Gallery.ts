import type { Component } from "svelte";

/**
 * Each file in `gallery/pages/` is one gallery page. It declares, in `<script module>`:
 * `export const meta: GalleryMeta = { title, group, order }`. Screens render inside a 1280 × 800 frame;
 * component sheets render at their natural height.
 */
export interface GalleryMeta {
    title: string;
    group: "Screens" | "Components";
    order: number;
    /** Screens draw inside the 1280 × 800 design frame. */
    frame?: boolean;
}

export interface GalleryPage {
    slug: string;
    meta: GalleryMeta;
    component: Component;
}

interface PageModule {
    default: Component;
    meta: GalleryMeta;
}

export class GalleryIndex {
    static load(): GalleryPage[] {
        const modules = import.meta.glob<PageModule>("./pages/*.svelte", { eager: true });
        return Object.entries(modules)
            .map(([path, mod]) => ({
                slug: path.replace(/^.*\/(.*)\.svelte$/, "$1"),
                meta: mod.meta,
                component: mod.default,
            }))
            .sort((a, b) => a.meta.group.localeCompare(b.meta.group) || a.meta.order - b.meta.order);
    }
}
