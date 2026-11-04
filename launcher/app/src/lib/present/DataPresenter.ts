import type { DataOverviewView } from "#lib/bindings/DataOverviewView.ts";
import type { WorldSourceView } from "#lib/bindings/WorldSourceView.ts";
import { ActivityTextSet } from "#lib/text/ActivityTextSet.ts";
import type { AdoptedInstance, ChoiceItem, SharedDataStats, WorldLocation, WorldSource } from "@amml/launcher-kit";
import type { AdoptView } from "#lib/bindings/AdoptView.ts";
import { InstancePresenter } from "./InstancePresenter.ts";
import { SwatchRules } from "./SwatchRules.ts";

// Turns the shared data folder's facts into the kit's stats, locations, sources and adopt models.
export class DataPresenter {
    static toStats(view: DataOverviewView): SharedDataStats {
        return {
            folder: InstancePresenter.shownFolder(view.folder),
            worlds: view.worlds,
            resourcePacks: view.resourcePacks,
            behaviorPacks: view.behaviorPacks,
            bytes: view.bytes,
        };
    }

    static toLocations(view: DataOverviewView): WorldLocation[] {
        return view.locations.map((location) => ({
            instanceId: location.instance,
            name: location.instance,
            swatch: SwatchRules.forName(location.instance),
            mode: location.mode,
            folder: location.folder === null ? undefined : InstancePresenter.shownFolder(location.folder),
        }));
    }

    // `names` words the two sources the bridge knows by id: the Store game's folder and an instance's
    static toSources(sources: readonly WorldSourceView[], names: { appdata: string }, now: number): WorldSource[] {
        return sources.map((source) => ({
            id: source.id,
            name: source.id === "appdata" ? names.appdata : source.label,
            folder: InstancePresenter.shownFolder(source.path),
            worldCount: source.worldCount,
            modified: source.modified === null ? "" : ActivityTextSet.ago(now - source.modified),
        }));
    }

    static toAdopted(views: readonly AdoptView[]): AdoptedInstance[] {
        return views.map((view) => ({
            id: view.name,
            name: view.name,
            swatch: SwatchRules.forName(view.name),
            ammlVersion: view.amml ?? undefined,
            modCount: view.modCount,
        }));
    }

    // The choices of the adopt step: each source of worlds, then a clean start.
    static toChoices(sources: readonly WorldSource[], scratch: { id: string; title: string; detail: string }): ChoiceItem[] {
        return [
            ...sources.map((source) => ({
                id: source.id,
                title: source.name,
                detail: source.folder,
                meta: DataPresenter.meta(source),
            })),
            { id: scratch.id, title: scratch.title, detail: scratch.detail },
        ];
    }

    private static meta(source: WorldSource): string {
        return source.modified === "" ? "" : source.modified;
    }
}
