import { listen } from "@tauri-apps/api/event";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";

// The registries as the background refresh stores them, sent each time a refresh ends.
export class RegistryEvents {
    static readonly EVENT = "launcher://registries";

    // Calls `receive` with each new list; the result stops the subscription.
    static subscribe(receive: (views: RegistryView[]) => void): Promise<() => void> {
        return listen<RegistryView[]>(RegistryEvents.EVENT, (event) => receive(event.payload));
    }
}
