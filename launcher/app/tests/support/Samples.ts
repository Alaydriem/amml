import type { CheckView } from "#lib/bindings/CheckView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";
import type { VersionView } from "#lib/bindings/VersionView.ts";

// Small facts for the store tests.
export class Samples {
    static instance(name: string, overrides: Partial<InstanceView> = {}): InstanceView {
        return {
            name,
            folder: `C:\\Users\\me\\AppData\\Local\\amml\\instances\\${name}`,
            game: "1.26.60.05",
            preview: false,
            ammlVersion: "26.60.1",
            ammlInstalled: true,
            modCount: 3,
            dataMode: "shared",
            dataDir: null,
            managed: true,
            runningSince: null,
            lastPlayed: null,
            updateCount: 0,
            ...overrides,
        };
    }

    static check(id: CheckView["id"], state: CheckView["state"], overrides: Partial<CheckView> = {}): CheckView {
        return { id, state, package: null, version: null, freeBytes: null, drive: null, action: null, elevated: false, ...overrides };
    }

    static version(version: string, channel: VersionView["channel"], state: VersionView["state"] = "download"): VersionView {
        return { version, channel, state, size: null, instanceCount: 0, amml: "26.60.1", defaultMods: 4 };
    }

    static job(id: number, state: JobSnapshot["state"], overrides: Partial<JobSnapshot> = {}): JobSnapshot {
        return {
            id,
            kind: "create-instance",
            titleName: "play",
            version: null,
            state,
            steps: [],
            stepNumber: 1,
            stepTotal: 1,
            progress: null,
            fault: null,
            started: 0,
            ...overrides,
        };
    }
}
