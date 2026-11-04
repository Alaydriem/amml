import { describe, expect, it, vi } from "vitest";
import { SetupStore, type SetupApi } from "#lib/stores/SetupStore.svelte.ts";
import { FakeEvents } from "../support/FakeEvents.ts";
import { Samples } from "../support/Samples.ts";

function setupApi(overrides: Partial<SetupApi> = {}): SetupApi {
    return {
        checksRun: vi.fn(async () => []),
        gameInputInstall: vi.fn(async () => 7),
        openUrl: vi.fn(async () => {}),
        adoptCandidates: vi.fn(async () => ({ instances: [], sources: [] })),
        setupRun: vi.fn(async () => 9),
        instancesList: vi.fn(async () => []),
        jobGet: vi.fn(async () => null),
        ...overrides,
    };
}

describe("SetupStore checks", () => {
    it("enables Next only when the Store and Gaming Services checks are ok", async () => {
        const navigate = vi.fn();
        const api = setupApi({
            checksRun: vi.fn(async () => [Samples.check("store", "ok"), Samples.check("gaming-services", "missing")]),
        });
        const store = new SetupStore(api, new FakeEvents(), navigate);
        await store.loadChecks();

        expect(store.canContinue).toBe(false);
        store.next();
        expect(navigate).not.toHaveBeenCalled();

        store.checks = [Samples.check("store", "ok"), Samples.check("gaming-services", "ok"), Samples.check("gameinput", "missing")];
        expect(store.canContinue).toBe(true);
        store.next();
        expect(navigate).toHaveBeenCalledWith("/setup/adopt");
    });

    it("lets Skip go on whatever the checks say", () => {
        const navigate = vi.fn();
        new SetupStore(setupApi(), new FakeEvents(), navigate).skip();
        expect(navigate).toHaveBeenCalledWith("/setup/adopt");
    });

    it("re-runs the checks after the GameInput job ends", async () => {
        const events = new FakeEvents();
        const checksRun = vi.fn(async () => [Samples.check("gameinput", "missing", { action: "install", elevated: true })]);
        const store = new SetupStore(setupApi({ checksRun }), events, vi.fn());
        await store.loadChecks();
        expect(checksRun).toHaveBeenCalledTimes(1);

        const acting = store.act("gameinput");
        await FakeEvents.settle();
        expect(checksRun).toHaveBeenCalledTimes(1);
        events.emit(Samples.job(99, "done"));
        await FakeEvents.settle();
        expect(checksRun).toHaveBeenCalledTimes(1);
        events.emit(Samples.job(7, "done", { kind: "install-game-input" }));
        await acting;

        expect(checksRun).toHaveBeenCalledTimes(2);
        expect(events.listeners).toBe(0);
    });

    it("shows the fault when the GameInput job fails, for example after a refused UAC prompt", async () => {
        const events = new FakeEvents();
        const store = new SetupStore(setupApi(), events, vi.fn());
        store.checks = [Samples.check("gameinput", "missing", { action: "install", elevated: true })];

        const acting = store.act("gameinput");
        await FakeEvents.settle();
        const fault = { kind: "operation-failed" as const, code: "the installer ended with exit code Some(1)", values: {}, restored: false };
        events.emit(Samples.job(7, "failed", { fault }));
        await acting;

        expect(store.fault?.code).toContain("installer");
    });

    it("opens the Store page of a missing app", async () => {
        const openUrl = vi.fn(async () => {});
        const store = new SetupStore(setupApi({ openUrl }), new FakeEvents(), vi.fn());
        store.checks = [Samples.check("gaming-services", "missing", { action: "open-store" })];

        await store.act("gaming-services");

        expect(openUrl).toHaveBeenCalledWith("ms-windows-store://pdp/?ProductId=9MWPM2CQNLHN");
    });
});

describe("SetupStore adopt", () => {
    const found = {
        instances: [
            { name: "a", game: "1.26.60.05", amml: null, modCount: 2 },
            { name: "b", game: "1.26.60.05", amml: "26.60.1", modCount: 0 },
        ],
        sources: [{ id: "appdata", label: "appdata", path: "C:\\x", worldCount: 3, modified: null }],
    };

    it("offers each source of worlds and a clean start, choosing the first source", async () => {
        const store = new SetupStore(setupApi({ adoptCandidates: vi.fn(async () => found) }), new FakeEvents(), vi.fn());
        await store.loadAdopt();

        expect(store.selected).toBe("appdata");
        expect(store.choices.map((choice) => choice.id)).toEqual(["appdata", "scratch"]);
        expect(store.adopted.map((instance) => instance.name)).toEqual(["a", "b"]);
    });

    it("runs the setup job for the chosen source and opens the list, or the new instance screen when there is none", async () => {
        for (const [existing, path] of [[1, "/instances"], [0, "/instances/new"]] as const) {
            const events = new FakeEvents();
            const navigate = vi.fn();
            const setupRun = vi.fn(async () => 9);
            const api = setupApi({
                adoptCandidates: vi.fn(async () => found),
                setupRun,
                instancesList: vi.fn(async () => Array.from({ length: existing }, (_, i) => Samples.instance(`i${i}`))),
            });
            const store = new SetupStore(api, events, navigate);
            await store.loadAdopt();
            store.select("scratch");

            const adopting = store.adopt();
            await FakeEvents.settle();
            events.emit(Samples.job(9, "done", { kind: "setup-data" }));
            await adopting;

            expect(setupRun).toHaveBeenCalledExactlyOnceWith("scratch");
            expect(navigate).toHaveBeenCalledWith(path);
            expect(store.busy).toBe(false);
        }
    });

    it("shows a failed setup job's fault and stays on the screen", async () => {
        const events = new FakeEvents();
        const navigate = vi.fn();
        const store = new SetupStore(setupApi({ adoptCandidates: vi.fn(async () => found) }), events, navigate);
        await store.loadAdopt();

        const adopting = store.adopt();
        await FakeEvents.settle();
        events.emit(Samples.job(9, "failed", { fault: { kind: "operation-failed", code: "boom", values: {}, restored: false } }));
        await adopting;

        expect(navigate).not.toHaveBeenCalled();
        expect(store.fault?.code).toBe("boom");
        // Next is available again
        expect(store.busy).toBe(false);
    });
});

describe("SetupStore failed setup", () => {
    it("shows a general fault when the job ends failed without one, and lets the player try again", async () => {
        const events = new FakeEvents();
        const navigate = vi.fn();
        const setupRun = vi.fn().mockResolvedValueOnce(9).mockResolvedValueOnce(10);
        const store = new SetupStore(setupApi({ setupRun, adoptCandidates: vi.fn(async () => ({ instances: [], sources: [] })) }), events, navigate);
        await store.loadAdopt();

        const first = store.adopt();
        await FakeEvents.settle();
        events.emit(Samples.job(9, "failed", { kind: "setup-data" }));
        await first;
        expect(store.fault?.kind).toBe("operation-failed");
        expect(store.busy).toBe(false);

        const second = store.adopt();
        await FakeEvents.settle();
        expect(store.fault).toBeNull();
        events.emit(Samples.job(10, "done", { kind: "setup-data" }));
        await second;
        expect(setupRun).toHaveBeenCalledTimes(2);
    });
});
