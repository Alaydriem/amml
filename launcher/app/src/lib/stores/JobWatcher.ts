import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { JobEvents } from "#lib/api/JobEvents.ts";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";

export type JobWatcherApi = Pick<typeof LauncherApi, "jobGet">;
export type JobWatcherEvents = Pick<typeof JobEvents, "subscribe">;

// Waits for one job to end. A job started a moment ago may have ended before the watch began, so the watcher asks
// for the job once after it subscribes, and it ignores the snapshots of any other job.
export class JobWatcher {
    static async untilEnded(
        id: number,
        api: JobWatcherApi,
        events: JobWatcherEvents,
        onSnapshot?: (snapshot: JobSnapshot) => void,
    ): Promise<JobSnapshot> {
        return new Promise<JobSnapshot>((resolve, reject) => {
            let stop: (() => void) | undefined;
            let finished = false;
            const accept = (snapshot: JobSnapshot) => {
                if (finished || snapshot.id !== id) return;
                onSnapshot?.(snapshot);
                if (snapshot.state === "running") return;
                finished = true;
                stop?.();
                resolve(snapshot);
            };
            events
                .subscribe(accept)
                .then(async (unlisten) => {
                    stop = unlisten;
                    if (finished) unlisten();
                    const current = await api.jobGet();
                    if (current) accept(current);
                })
                .catch(reject);
        });
    }
}
