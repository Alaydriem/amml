import type { VersionView } from "#lib/bindings/VersionView.ts";
import type { GameVersion } from "@amml/launcher-kit";

// Turns a game version's facts into the kit's `GameVersion` model.
export class VersionPresenter {
    static toGameVersion(view: VersionView): GameVersion {
        return {
            version: view.version,
            channel: view.channel,
            state: view.state,
            size: view.size ?? undefined,
            instanceCount: view.instanceCount,
        };
    }
}
