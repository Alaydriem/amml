import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ApiFault } from "#lib/api/ApiFault.ts";
import { InstancesStore, type InstancesApi } from "#lib/stores/InstancesStore.svelte.ts";
import { Samples } from "../support/Samples.ts";

function api(list: () => ReturnType<InstancesApi["instancesList"]>, overrides: Partial<InstancesApi> = {}): InstancesApi {
    return {
        instancesList: vi.fn(list),
        instanceLaunch: vi.fn(async () => {}),
        instanceDelete: vi.fn(async () => {}),
        openPath: vi.fn(async () => {}),
        ...overrides,
    };
}

describe("InstancesStore", () => {
    beforeEach(() => vi.useFakeTimers());
    afterEach(() => vi.useRealTimers());

    it("polls only while an instance runs", async () => {
        let running = true;
        const calls = vi.fn();
        const store = new InstancesStore(
            api(async () => {
                calls();
                return [Samples.instance("play", { runningSince: running ? 1 : null })];
            }),
            vi.fn(),
        );

        await store.load();
        expect(calls).toHaveBeenCalledTimes(1);
        await vi.advanceTimersByTimeAsync(InstancesStore.POLL_MILLIS * 3);
        expect(calls).toHaveBeenCalledTimes(4);

        running = false;
        await vi.advanceTimersByTimeAsync(InstancesStore.POLL_MILLIS);
        const stopped = calls.mock.calls.length;
        await vi.advanceTimersByTimeAsync(InstancesStore.POLL_MILLIS * 5);
        expect(calls).toHaveBeenCalledTimes(stopped);
        store.dispose();
    });

    it("does not poll when no instance runs", async () => {
        const calls = vi.fn();
        const store = new InstancesStore(
            api(async () => {
                calls();
                return [Samples.instance("play")];
            }),
            vi.fn(),
        );

        await store.load();
        await vi.advanceTimersByTimeAsync(InstancesStore.POLL_MILLIS * 5);

        expect(calls).toHaveBeenCalledTimes(1);
    });

    it("deletes only after the confirmation, with the name as the confirm text", async () => {
        const instanceDelete = vi.fn(async () => {});
        const store = new InstancesStore(api(async () => [Samples.instance("play")], { instanceDelete }), vi.fn());
        await store.load();

        await store.menuAction("play", "delete");
        expect(store.pendingDelete?.name).toBe("play");
        expect(instanceDelete).not.toHaveBeenCalled();

        store.cancelDelete();
        await store.confirmDelete();
        expect(instanceDelete).not.toHaveBeenCalled();

        await store.menuAction("play", "delete");
        await store.confirmDelete();
        expect(instanceDelete).toHaveBeenCalledExactlyOnceWith("play", "play");
        expect(store.pendingDelete).toBeNull();
    });

    it("opens the folder of an instance and the page of the first instance with updates", async () => {
        const openPath = vi.fn(async () => {});
        const navigate = vi.fn();
        const store = new InstancesStore(
            api(async () => [Samples.instance("one"), Samples.instance("two", { updateCount: 2 })], { openPath }),
            navigate,
        );
        await store.load();

        await store.menuAction("one", "open-folder");
        store.review();
        store.open("a b");
        store.newInstance();

        expect(openPath).toHaveBeenCalledWith("C:\\Users\\me\\AppData\\Local\\amml\\instances\\one");
        expect(navigate.mock.calls.map((call) => call[0])).toEqual(["/instances/two", "/instances/a%20b", "/instances/new"]);
    });

    it("shows the fault of a refused launch and keeps the list", async () => {
        const refused = { kind: "instance-running" as const, code: null, values: { name: "play" }, restored: false };
        const store = new InstancesStore(
            api(async () => [Samples.instance("play")], {
                instanceLaunch: vi.fn(async () => {
                    throw new ApiFault(refused);
                }),
            }),
            vi.fn(),
        );
        await store.load();

        await store.launch("play");

        expect(store.fault?.kind).toBe("instance-running");
        expect(store.views).toHaveLength(1);
    });

    it("words each instance for the kit", async () => {
        const store = new InstancesStore(api(async () => [Samples.instance("play", { ammlVersion: null })]), vi.fn(), () => 0);
        await store.load();

        const [shown] = store.instances();

        expect(shown?.ammlVersion).toBe("unknown version");
        expect(shown?.folder).toBe("%LOCALAPPDATA%\\amml\\instances\\play");
        expect(shown?.activity).toBe("never played");
    });
});
