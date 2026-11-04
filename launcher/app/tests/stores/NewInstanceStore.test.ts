import { describe, expect, it, vi } from "vitest";
import { NewInstanceStore, type NewInstanceApi } from "#lib/stores/NewInstanceStore.svelte.ts";
import { Samples } from "../support/Samples.ts";

function api(overrides: Partial<NewInstanceApi> = {}): NewInstanceApi {
    return {
        versionsList: vi.fn(async (channel) =>
            channel === "release"
                ? [Samples.version("1.21.0.03", "release", "unsupported"), Samples.version("1.26.60.05", "release")]
                : [Samples.version("1.26.70.20", "preview")],
        ),
        settingsGet: vi.fn(async () => ({ preview: false, dataDir: "d", logsDir: "l" })),
        instancesList: vi.fn(async () => [Samples.instance("taken")]),
        nameProblem: vi.fn(async () => null),
        instanceCreate: vi.fn(async () => 5),
        ...overrides,
    };
}

describe("NewInstanceStore", () => {
    it("hides the preview channel unless preview builds are on", async () => {
        const off = new NewInstanceStore(api(), vi.fn());
        await off.load();
        expect(off.channels()).toEqual(["release"]);
        await off.setChannel("preview");
        expect(off.channel).toBe("release");

        const on = new NewInstanceStore(
            api({ settingsGet: vi.fn(async () => ({ preview: true, dataDir: "d", logsDir: "l" })) }),
            vi.fn(),
        );
        await on.load();
        expect(on.channels()).toEqual(["release", "preview"]);
        await on.setChannel("preview");
        expect(on.channel).toBe("preview");
        expect(on.selectedVersion).toBe("1.26.70.20");
    });

    it("picks the first version a build supports and reads the instance names", async () => {
        const store = new NewInstanceStore(api(), vi.fn());
        await store.load();

        expect(store.selectedVersion).toBe("1.26.60.05");
        expect(store.takenNames).toEqual(["taken"]);
        expect(store.summary).toEqual({ ammlVersion: "26.60.1", defaultMods: 4, diskBytes: NewInstanceStore.TYPICAL_BYTES });
    });

    it("create_stays_disabled_while_the_name_has_a_problem", async () => {
        const instanceCreate = vi.fn(async () => 5);
        const navigate = vi.fn();
        const nameProblem = vi.fn(async (name: string) => (name === "taken" ? ("taken" as const) : null));
        const store = new NewInstanceStore(api({ instanceCreate, nameProblem }), navigate);
        await store.load();

        await store.setName("taken");
        expect(store.canCreate).toBe(false);
        expect(store.kitTakenNames).toContain("taken");
        await store.create({ name: "taken", version: "1.26.60.05", dataMode: "shared" });
        expect(instanceCreate).not.toHaveBeenCalled();
        expect(navigate).not.toHaveBeenCalled();

        await store.setName("fresh");
        expect(store.canCreate).toBe(true);
        expect(store.kitTakenNames).toEqual(["taken"]);
    });

    it("tells why a bad name is refused and holds Create back, while empty and taken need no extra line", async () => {
        const problems: Record<string, "bad" | "empty" | "taken"> = { con: "bad", "": "empty", taken: "taken" };
        const nameProblem = vi.fn(async (name: string) => problems[name] ?? null);
        const instanceCreate = vi.fn(async () => 5);
        const store = new NewInstanceStore(api({ nameProblem, instanceCreate }), vi.fn());
        await store.load();

        await store.setName("con");
        expect(store.problemText).toContain("Use another name.");
        expect(store.problemText).toContain("Windows device name such as CON.");
        expect(store.canCreate).toBe(false);
        await store.create({ name: "con", version: "1.26.60.05", dataMode: "shared" });
        expect(instanceCreate).not.toHaveBeenCalled();

        await store.setName("");
        expect(store.problemText).toBeUndefined();
        expect(store.canCreate).toBe(false);
        await store.setName("taken");
        expect(store.problemText).toBeUndefined();
        expect(store.kitTakenNames).toContain("taken");
        await store.setName("fresh");
        expect(store.problemText).toBeUndefined();
        expect(store.canCreate).toBe(true);
    });

    it("creates the instance and opens the install screen", async () => {
        const instanceCreate = vi.fn(async () => 5);
        const navigate = vi.fn();
        const store = new NewInstanceStore(api({ instanceCreate }), navigate);
        await store.load();
        await store.setName("fresh");

        await store.create({ name: "fresh", version: "1.26.60.05", dataMode: "own" });

        expect(instanceCreate).toHaveBeenCalledWith("fresh", "1.26.60.05", false, true);
        expect(navigate).toHaveBeenCalledWith("/instances/install");
    });

    it("keeps the answer of the newest name when an older answer comes late", async () => {
        let release: (value: "bad" | null) => void = () => {};
        const late = new Promise<"bad" | null>((resolve) => (release = resolve));
        const nameProblem = vi.fn().mockReturnValueOnce(late).mockResolvedValueOnce(null);
        const store = new NewInstanceStore(api({ nameProblem }), vi.fn());
        await store.load();

        const first = store.setName("co");
        await store.setName("con-fixed");
        release("bad");
        await first;

        expect(store.problem).toBeNull();
    });
});
