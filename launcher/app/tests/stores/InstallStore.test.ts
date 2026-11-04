import { describe, expect, it, vi } from "vitest";
import { InstallStore, type InstallApi } from "#lib/stores/InstallStore.svelte.ts";
import { FakeEvents } from "../support/FakeEvents.ts";
import { Samples } from "../support/Samples.ts";

function api(current: ReturnType<typeof Samples.job> | null, overrides: Partial<InstallApi> = {}): InstallApi {
    return {
        jobGet: vi.fn(async () => current),
        jobCancel: vi.fn(async () => {}),
        jobRetry: vi.fn(async () => 2),
        jobRetryDeveloperMode: vi.fn(async () => 3),
        openPath: vi.fn(async () => {}),
        ...overrides,
    };
}

const running = Samples.job(1, "running", {
    version: "1.26.51.1",
    steps: [
        { id: "version-check", state: "done", millis: 400, cached: false, percent: null },
        { id: "version-download-current", state: "done", millis: null, cached: true, percent: null },
        { id: "version-download-wanted", state: "active", millis: null, cached: false, percent: 62 },
        { id: "version-install", state: "waiting", millis: null, cached: false, percent: null },
    ],
    stepNumber: 3,
    stepTotal: 4,
    progress: { done: 1_280_000_000, total: 2_060_000_000, rate: 38_000_000, remainingSeconds: 20, file: "1.26.51.1.msixvc", mirror: null },
});

describe("InstallStore", () => {
    it("the_install_store_shows_the_running_job_snapshot_on_mount", async () => {
        const store = new InstallStore(api(running), new FakeEvents(), vi.fn(), async () => {});

        await store.start();

        expect(store.loaded).toBe(true);
        expect(store.steps.map((step) => [step.label, step.state, step.note])).toEqual([
            ["Check the Store version", "done", "0.4 s"],
            ["Downloading your installed version", "done", "cached"],
            ["Download 1.26.51.1", "active", "62%"],
            ["Installing 1.26.51.1", "waiting", undefined],
        ]);
        expect(store.progress).toEqual({
            done: 1_280_000_000,
            total: 2_060_000_000,
            rate: 38_000_000,
            remaining: 20,
            detail: "1.26.51.1.msixvc",
        });
        expect(store.snapshot?.stepNumber).toBe(3);
        expect(store.fault).toBeUndefined();
    });

    it("follows the job's snapshots and ignores an older job's", async () => {
        const events = new FakeEvents();
        const store = new InstallStore(api(running), events, vi.fn(), async () => {});
        await store.start();

        events.emit({ ...running, stepNumber: 4 });
        expect(store.snapshot?.stepNumber).toBe(4);
        events.emit(Samples.job(0, "running", { stepNumber: 1 }));
        expect(store.snapshot?.stepNumber).toBe(4);
        store.stopWatching();
        expect(events.listeners).toBe(0);
    });

    it("returns to the instance list when the job is cancelled", async () => {
        const navigate = vi.fn();
        const events = new FakeEvents();
        const store = new InstallStore(api(running), events, navigate, async () => {});
        await store.start();

        await store.cancel();
        events.emit(Samples.job(1, "cancelled"));

        expect(navigate).toHaveBeenCalledWith("/instances");
    });

    it("goes to the new instance when a create is done and back to the instance after other jobs", async () => {
        const navigate = vi.fn();
        const events = new FakeEvents();
        const store = new InstallStore(api(running), events, navigate, async () => {});
        await store.start();

        events.emit(Samples.job(1, "done", { titleName: "a b" }));
        expect(navigate).toHaveBeenLastCalledWith("/instances/a%20b");
        expect(InstallStore.returnTo(Samples.job(2, "done", { kind: "copy-worlds", titleName: "appdata" }))).toBe("/settings/shared");
        expect(InstallStore.returnTo(Samples.job(2, "done", { kind: "install-game-input", titleName: "" }))).toBe("/instances");
    });

    it("shows a failed job with its fault, the restored note and a retry that starts the job again", async () => {
        const jobRetry = vi.fn(async () => 2);
        const events = new FakeEvents();
        const fault = { kind: "download-failed" as const, code: null, values: { done: "1280000000", total: "2060000000" }, restored: true };
        const store = new InstallStore(api(running, { jobRetry }), events, vi.fn(), async () => {});
        await store.start();

        events.emit(Samples.job(1, "failed", { fault, steps: running.steps }));

        expect(store.fault?.kind).toBe("download-failed");
        expect(store.fault?.text.body).toBe("1.3 GB of 2.1 GB is kept and the download continues from there.");
        expect(store.snapshot?.fault?.restored).toBe(true);
        await store.retry();
        expect(jobRetry).toHaveBeenCalledOnce();
    });

    it("shows a cancelling state from the press of Cancel until the job stops", async () => {
        const events = new FakeEvents();
        const store = new InstallStore(api(running), events, vi.fn(), async () => {});
        await store.start();
        expect(store.cancelling).toBe(false);

        await store.cancel();
        expect(store.cancelling).toBe(true);
        events.emit({ ...running, stepNumber: 4 });
        expect(store.cancelling).toBe(true);
        events.emit(Samples.job(1, "cancelled"));
        expect(store.cancelling).toBe(false);
    });

    it("leaves the cancelling state when the cancel call throws", async () => {
        const jobCancel = vi.fn(async () => {
            throw { kind: "unavailable", values: {} };
        });
        const store = new InstallStore(api(running, { jobCancel }), new FakeEvents(), vi.fn(), async () => {});
        await store.start();

        await store.cancel();

        expect(jobCancel).toHaveBeenCalledOnce();
        expect(store.cancelling).toBe(false);
        expect(store.failure).not.toBeNull();
    });

    it("words the step a job shows while it waits for another task", async () => {
        const waiting = Samples.job(1, "running", {
            steps: [
                { id: "manager-wait", state: "active", millis: null, cached: false, percent: null },
                { id: "amml-download", state: "waiting", millis: null, cached: false, percent: null },
            ],
            stepNumber: 1,
            stepTotal: 2,
        });
        const store = new InstallStore(api(waiting), new FakeEvents(), vi.fn(), async () => {});

        await store.start();

        expect(store.steps.map((step) => [step.label, step.state])).toEqual([
            ["Waiting for another task to end", "active"],
            ["Download AMML", "waiting"],
        ]);
    });

    it("opens the Developer Mode dialog on consent-needed and retries on yes", async () => {
        const jobRetryDeveloperMode = vi.fn(async () => 3);
        const events = new FakeEvents();
        const consent = { kind: "consent-needed" as const, code: null, values: { version: "1.26.51.1" }, restored: true };
        const store = new InstallStore(api(running, { jobRetryDeveloperMode }), events, vi.fn(), async () => {});
        await store.start();

        events.emit(Samples.job(1, "failed", { fault: consent }));

        expect(store.asking).toBe(true);
        expect(store.fault).toBeUndefined();
        expect(store.developerModeValues).toEqual({ file: "Minecraft.Windows.exe", version: "1.26.51.1" });
        await store.allowDeveloperMode();
        expect(jobRetryDeveloperMode).toHaveBeenCalledOnce();
        expect(store.asking).toBe(false);
    });

    it("shows the copy failure when the player says no to Developer Mode", async () => {
        const events = new FakeEvents();
        const consent = { kind: "consent-needed" as const, code: null, values: {}, restored: true };
        const store = new InstallStore(api(running), events, vi.fn(), async () => {});
        await store.start();
        events.emit(Samples.job(1, "failed", { fault: consent }));

        store.refuseDeveloperMode();

        expect(store.asking).toBe(false);
        expect(store.fault?.kind).toBe("copy-failed");
        expect(store.fault?.text.title).toBe("Copying the game files failed.");
        expect(store.snapshot?.fault?.restored).toBe(true);
    });
});
