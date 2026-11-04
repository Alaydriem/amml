import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { ActivityTextSet } from "#lib/text/ActivityTextSet.ts";
import type { RegistrySource } from "@amml/launcher-kit";

// Turns a registry's facts into the kit's `RegistrySource` model.
export class RegistryPresenter {
    static toSource(view: RegistryView, now: number): RegistrySource {
        return {
            id: view.id,
            name: view.name ?? RegistryPresenter.shownUrl(view.base),
            url: RegistryPresenter.shownUrl(view.base),
            builtIn: view.official,
            modCount: view.modCount ?? undefined,
            status: RegistryPresenter.status(view),
            fetched: view.fetched === null ? undefined : ActivityTextSet.ago(now - view.fetched),
        };
    }

    // a registry that answered from its saved copy still has a list, so it shows as ok
    private static status(view: RegistryView): RegistrySource["status"] {
        switch (view.status) {
            case "ok":
            case "stale":
                return "ok";
            case "offline":
                return "offline";
            case "broken":
                return "failed";
            case "loading":
                return "loading";
        }
    }

    // the address without the scheme
    private static shownUrl(base: string): string {
        return base.replace(/^https?:\/\//, "");
    }
}
