import { describe, expect, it, vi } from "vitest";
import { SetupProgressStore, type SetupProgressApi } from "#lib/stores/SetupProgressStore.svelte.ts";
import { FakeEvents } from "../support/FakeEvents.ts";
import { Samples } from "../support/Samples.ts";

function api(current: ReturnType<typeof Samples.job> | null, overrides: Partial<SetupProgressApi> = {}): SetupProgressApi {
    return {
        jobGet: vi.fn(async () => current),
        jobRetry: vi.fn(async () => 2),
        instancesList: vi.fn(async () => []),
        openPath: vi.fn(async () => {}),
        ...overrides,
    };
}

const copying = Samples.job(1, "running", {
    kind: "setup-data",
    titleName: "appdata",
    steps: [
        { id: "setup-check", state: "done", millis: 400, cached: false, percent: null },
        { id: "setup-measure", state: "done", millis: 200, cached: false, percent: null },
        { id: "setup-copy", state: "active", millis: null, cached: false, percent: 40 },
        { id: "setup-settings", state: "waiting", millis: null, cached: false, percent: null },
    ],
    progress: { done: 10, total: 100, rate: null, remainingSeconds: null, file: "a → b", mirror: null },
});

describe("SetupProgressStore", () => {
    it("with no job goes to the adopt step", async () => {
        const navigate = vi.fn();
        const store = new SetupProgressStore(api(null), new FakeEvents(), navigate, async () => {});

        await store.start();

        expect(navigate).toHaveBeenCalledWith("/setup/adopt");
    });

    it("with a job of another kind goes to the adopt step", async () => {
        const navigate = vi.fn();
        const store = new SetupProgressStore(api(Samples.job(1, "running")), new FakeEvents(), navigate, async () => {});

        await store.start();

        expect(navigate).toHaveBeenCalledWith("/setup/adopt");
    });

    it("shows the steps and the readout of the running job", async () => {
        const store = new SetupProgressStore(api(copying), new FakeEvents(), vi.fn(), async () => {});

        await store.start();

        expect(store.steps.map((step) => [step.label, step.state])).toEqual([
            ["Check the shared data folder", "done"],
            ["Measure the worlds", "done"],
            ["Copy into shared data", "active"],
            ["Save launcher settings", "waiting"],
        ]);
        expect(store.progress?.detail).toBe("a → b");
    });

    it("done goes to new instance when there are no instances", async () => {
        const navigate = vi.fn();
        const events = new FakeEvents();
        const store = new SetupProgressStore(api(copying), events, navigate, async () => {});
        await store.start();

        events.emit({ ...copying, state: "done" });
        await FakeEvents.settle();

        expect(navigate).toHaveBeenCalledExactlyOnceWith("/instances/new");
    });

    it("done goes to the list when there are instances", async () => {
        const navigate = vi.fn();
        const events = new FakeEvents();
        const instancesList = vi.fn(async () => [Samples.instance("a")]);
        const store = new SetupProgressStore(api(copying, { instancesList }), events, navigate, async () => {});
        await store.start();

        events.emit({ ...copying, state: "done" });
        await FakeEvents.settle();

        expect(navigate).toHaveBeenCalledExactlyOnceWith("/instances");
    });

    it("failed shows the fault, and retry calls jobRetry", async () => {
        const failed = {
            ...copying,
            state: "failed" as const,
            fault: { kind: "copy-failed" as const, code: null, values: {}, restored: true },
        };
        const jobRetry = vi.fn(async () => 2);
        const store = new SetupProgressStore(api(failed, { jobRetry }), new FakeEvents(), vi.fn(), async () => {});

        await store.start();
        expect(store.fault).toBeDefined();

        await store.retry();
        expect(jobRetry).toHaveBeenCalledTimes(1);
    });

    it("picks the nothing-changed text for a fresh failure and the files-stay text for a merge failure", async () => {
        const failure = (restored: boolean) => ({
            ...copying,
            state: "failed" as const,
            fault: { kind: "copy-failed" as const, code: null, values: {}, restored },
        });
        const fresh = new SetupProgressStore(api(failure(true)), new FakeEvents(), vi.fn(), async () => {});
        await fresh.start();
        const merge = new SetupProgressStore(api(failure(false)), new FakeEvents(), vi.fn(), async () => {});
        await merge.start();

        expect(fresh.text.restored).toContain("Nothing was changed");
        expect(merge.text.restored).toContain("The files copied so far stay in shared data.");
        expect(merge.text.restored).toContain("Try again copies the rest.");
    });

    it("names the AppData source Store Minecraft and an instance source by its folder", async () => {
        const store = new SetupProgressStore(api(copying), new FakeEvents(), vi.fn(), async () => {});
        await store.start();
        expect(store.sourceName).toBe("Store Minecraft");

        const other = new SetupProgressStore(
            api({ ...copying, titleName: "1.26.60.05" }),
            new FakeEvents(),
            vi.fn(),
            async () => {},
        );
        await other.start();
        expect(other.sourceName).toBe("1.26.60.05");
    });

    it("chooseAnother goes to the adopt step", () => {
        const navigate = vi.fn();
        new SetupProgressStore(api(null), new FakeEvents(), navigate, async () => {}).chooseAnother();
        expect(navigate).toHaveBeenCalledWith("/setup/adopt");
    });
});
