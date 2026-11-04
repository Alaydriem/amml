import { listen } from "@tauri-apps/api/event";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";

// The running job's snapshots, as the bridge sends them.
export class JobEvents {
    static readonly EVENT = "launcher://job";

    // Calls `receive` with each snapshot; the result stops the subscription.
    static subscribe(receive: (snapshot: JobSnapshot) => void): Promise<() => void> {
        return listen<JobSnapshot>(JobEvents.EVENT, (event) => receive(event.payload));
    }
}
