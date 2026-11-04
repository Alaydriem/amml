import { describe, expect, it, vi } from "vitest";
import type { ModView } from "#lib/bindings/ModView.ts";
import { InstanceDetailStore, type InstanceDetailApi } from "#lib/stores/InstanceDetailStore.svelte.ts";
import { Samples } from "../support/Samples.ts";

function mod(id: string, overrides: Partial<ModView> = {}): ModView {
    return {
        id,
        title: id,
        origin: "registry",
        registryBase: "https://r/",
        enabled: true,
        installedVersion: "1.0.0",
        availableVersion: "1.0.0",
        api: 2,
        update: "upToDate",
        ...overrides,
    };
}

function api(mods: ModView[], overrides: Partial<InstanceDetailApi> = {}): InstanceDetailApi {
    return {
        instanceGet: vi.fn(async () => Samples.instance("play")),
        modsList: vi.fn(async () => mods),
        modSetEnabled: vi.fn(async () => {}),
        modRemove: vi.fn(async () => {}),
        modsUpdate: vi.fn(async () => 4),
        ammlUpdate: vi.fn(async () => 5),
        openPath: vi.fn(async () => {}),
        instanceLaunch: vi.fn(async () => {}),
        instanceDelete: vi.fn(async () => {}),
        ...overrides,
    };
}

function store(mods: ModView[], overrides: Partial<InstanceDetailApi> = {}, navigate = vi.fn()) {
    return new InstanceDetailStore("play", api(mods, overrides), navigate, () => []);
}

describe("InstanceDetailStore", () => {
    const mods = [
        mod("alaydriem/amml", { origin: "core", registryBase: null, update: "available", availableVersion: "26.60.2" }),
        mod("@alaydriem/peek", { update: "available", availableVersion: "1.1.0" }),
        mod("@alaydriem/zoom", { update: "available", availableVersion: "2.0.0" }),
        mod("@alaydriem/map"),
        mod("@x/url", { origin: "url", registryBase: null, update: "unofficial", availableVersion: null }),
    ];

    it("update all installs every available update and the AMML row last", async () => {
        const modsUpdate = vi.fn(async () => 4);
        const navigate = vi.fn();
        const detail = store(mods, { modsUpdate }, navigate);
        await detail.load();

        expect(detail.updateAllIds()).toEqual(["@alaydriem/peek", "@alaydriem/zoom", "alaydriem/amml"]);
        await detail.updateAll();

        expect(modsUpdate).toHaveBeenCalledExactlyOnceWith("play", ["@alaydriem/peek", "@alaydriem/zoom", "alaydriem/amml"]);
        expect(navigate).toHaveBeenCalledWith("/instances/install");
    });

    it("updates the AMML row through the AMML call and any other row through its own id", async () => {
        const modsUpdate = vi.fn(async () => 4);
        const ammlUpdate = vi.fn(async () => 5);
        const detail = store(mods, { modsUpdate, ammlUpdate });
        await detail.load();

        await detail.update("alaydriem/amml");
        await detail.update("@alaydriem/peek");

        expect(ammlUpdate).toHaveBeenCalledExactlyOnceWith("play");
        expect(modsUpdate).toHaveBeenCalledExactlyOnceWith("play", ["@alaydriem/peek"]);
    });

    it("does nothing for update all when no update waits", async () => {
        const modsUpdate = vi.fn(async () => 4);
        const detail = store([mod("@a/b")], { modsUpdate });
        await detail.load();
        await detail.updateAll();
        expect(modsUpdate).not.toHaveBeenCalled();
    });

    it("turns a mod on or off and reads the list again", async () => {
        const modSetEnabled = vi.fn(async () => {});
        const modsList = vi.fn(async () => mods);
        const detail = store(mods, { modSetEnabled, modsList });
        await detail.load();

        await detail.toggle("@alaydriem/peek", false);

        expect(modSetEnabled).toHaveBeenCalledWith("play", "@alaydriem/peek", false);
        expect(modsList).toHaveBeenCalledTimes(2);
    });

    it("removes a mod only after the confirmation", async () => {
        const modRemove = vi.fn(async () => {});
        const detail = store(mods, { modRemove });
        await detail.load();

        await detail.menuAction("@alaydriem/zoom", "remove");
        expect(detail.pendingRemove?.id).toBe("@alaydriem/zoom");
        expect(modRemove).not.toHaveBeenCalled();
        detail.cancelRemove();
        await detail.confirmRemove();
        expect(modRemove).not.toHaveBeenCalled();

        await detail.menuAction("@alaydriem/zoom", "remove");
        await detail.confirmRemove();
        expect(modRemove).toHaveBeenCalledExactlyOnceWith("play", "@alaydriem/zoom");
    });

    it("opens the folder of an enabled mod and of a disabled one", async () => {
        const openPath = vi.fn(async () => {});
        const detail = store([mod("@a/on"), mod("@a/off", { enabled: false })], { openPath });
        await detail.load();

        await detail.menuAction("@a/on", "folder");
        await detail.menuAction("@a/off", "folder");

        const folder = "C:\\Users\\me\\AppData\\Local\\amml\\instances\\play";
        expect((openPath.mock.calls as unknown as string[][]).map((call) => call[0])).toEqual([
            `${folder}\\mods\\amml\\@a\\on`,
            `${folder}\\mods\\amml-disabled\\@a\\off`,
        ]);
    });

    it("deletes the instance only after the confirmation, with its name, and goes to the list", async () => {
        const instanceDelete = vi.fn(async () => {});
        const navigate = vi.fn();
        const detail = store(mods, { instanceDelete }, navigate);
        await detail.load();

        detail.askDelete();
        expect(instanceDelete).not.toHaveBeenCalled();
        await detail.confirmDelete();

        expect(instanceDelete).toHaveBeenCalledExactlyOnceWith("play", "play");
        expect(navigate).toHaveBeenCalledWith("/instances");
    });

    it("shows a busy launcher as a fault and keeps what it has", async () => {
        const busy = { kind: "busy" as const, code: null, values: {}, restored: false };
        const detail = store(mods, {
            modsList: vi.fn(async () => {
                throw busy;
            }),
        });

        await detail.load();

        expect(detail.fault?.kind).toBe("busy");
        expect(detail.loaded).toBe(true);
    });

    it("words the mods and the header for the kit", async () => {
        const detail = store(mods);
        await detail.load();

        expect(detail.installed().map((held) => held.origin)).toEqual(["core", "registry", "registry", "registry", "url"]);
        expect(detail.header()?.name).toBe("play");
    });
});
