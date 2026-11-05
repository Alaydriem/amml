import type { ModView } from "#lib/bindings/ModView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import type { ModNoteText } from "#lib/text/ModNoteTextSet.ts";
import type { InstalledMod, ModOrigin } from "@amml/launcher-kit";

// Turns a mod's facts into the kit's `InstalledMod` model.
export class ModPresenter {
    static readonly AMML_ID = "alaydriem/amml";

    // The kit has three origins; a mod found on disk shows as `url` with the note "local".
    static toInstalled(view: ModView, registries: readonly RegistryView[], notes: ModNoteText): InstalledMod {
        const registry = registries.find((held) => held.base === view.registryBase);
        const origin: ModOrigin = view.origin === "local" ? "url" : view.origin;
        const localBuild = view.update === "localBuild";
        const hasAvailable = view.update === "available" || view.update === "upToDate" || localBuild;
        return {
            id: view.id,
            title: view.id === ModPresenter.AMML_ID ? notes.loaderTitle : (view.title ?? ModPresenter.nameOf(view.id)),
            origin,
            registryName: origin === "registry" ? (registry?.name ?? view.registryBase ?? undefined) : undefined,
            enabled: view.enabled,
            installedVersion: view.installedVersion ?? "",
            availableVersion: hasAvailable ? (view.availableVersion ?? undefined) : undefined,
            note: ModPresenter.noteOf(view, registry, notes),
            localBuild,
        };
    }

    private static noteOf(view: ModView, registry: RegistryView | undefined, notes: ModNoteText): string | undefined {
        if (view.id === ModPresenter.AMML_ID) return undefined;
        if (view.origin === "core") return notes.coreAlwaysOn;
        if (view.origin === "local") return notes.local;
        if (view.origin === "url") return notes.fromUrl;
        return registry?.official ? notes.official : undefined;
    }

    // the part of `@namespace/name` after the slash
    private static nameOf(id: string): string {
        return id.split("/")[1] ?? id;
    }
}
