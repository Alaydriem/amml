import type { RegistryEvents } from "#lib/api/RegistryEvents.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";

export type RegistryFeedEvents = Pick<typeof RegistryEvents, "subscribe">;

// Follows the stored registries for one page: it listens first, then reads the stored list, and a list from an
// event always wins over a read that ends after it.
export class RegistryFeed {
    private readonly events: RegistryFeedEvents;
    private readonly onViews: (views: RegistryView[]) => void;
    private stop: (() => void) | undefined;
    private subscribing: Promise<void> | undefined;
    private stopped = false;
    // counts the lists that events brought
    private arrived = 0;

    constructor(events: RegistryFeedEvents, onViews: (views: RegistryView[]) => void) {
        this.events = events;
        this.onViews = onViews;
    }

    // Starts listening once, then reads the stored list. Gives the list to `onViews` unless an event came first.
    async load(read: () => Promise<RegistryView[]>): Promise<void> {
        await this.listen();
        const before = this.arrived;
        const views = await read();
        if (this.arrived === before && !this.stopped) this.onViews(views);
    }

    stopWatching(): void {
        this.stopped = true;
        this.stop?.();
        this.stop = undefined;
    }

    private listen(): Promise<void> {
        this.subscribing ??= this.events
            .subscribe((views) => {
                if (this.stopped) return;
                this.arrived += 1;
                this.onViews(views);
            })
            .then(
                (stop) => {
                    if (this.stopped) stop();
                    else this.stop = stop;
                },
                () => {
                    // without events the stored list is still read
                },
            );
        return this.subscribing;
    }
}
