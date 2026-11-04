import { describe, expect, it, vi } from "vitest";
import { ApiFault } from "#lib/api/ApiFault.ts";
import type { AboutView } from "#lib/bindings/AboutView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { SettingsStore, type SettingsApi } from "#lib/stores/SettingsStore.svelte.ts";
import { Samples } from "../support/Samples.ts";

const registry = (id: string, official = false): RegistryView => ({
    id,
    base: `https://${id}/`,
    name: id,
    official,
    status: "ok",
    modCount: 3,
    fetched: 0,
});

const about: AboutView = {
    launcherVersion: "0.1.0",
    builtOn: "2026-11-02",
    windowsVersion: "10.0.26300.7171",
    storePackage: "Microsoft.MinecraftUWP_1",
    storeVersion: "1.26.60.5",
    developerMode: true,
    registry: registry("official", true),
    logsDir: "C:\\Users\\me\\AppData\\Local\\amml\\launcher\\logs",
};

function api(overrides: Partial<SettingsApi> = {}): SettingsApi {
    return {
        settingsGet: vi.fn(async () => ({
            preview: false,
                        dataDir: "C:\\d",
            logsDir: "C:\\Users\\me\\AppData\\Local\\amml\\launcher\\logs",
        })),
        settingsSetPreview: vi.fn(async () => {}),
        registriesRefresh: vi.fn(async () => [registry("official", true), registry("friends")]),
        registryAdd: vi.fn(async () => {}),
        registryRemove: vi.fn(async () => {}),
        versionsOnDisk: vi.fn(async () => [Samples.version("1.26.60.05", "release", "installed")]),
        versionsList: vi.fn(async () => [Samples.version("1.26.60.05", "release")]),
        versionRemove: vi.fn(async () => true),
        dataOverview: vi.fn(async () => ({
            folder: "C:\\d\\Minecraft Bedrock",
            worlds: 2,
            resourcePacks: 1,
            behaviorPacks: 0,
            bytes: 5,
            locations: [{ instance: "play", mode: "shared" as const, folder: null }],
            sources: [{ id: "appdata", label: "appdata", path: "C:\\a", worldCount: 3, modified: null }],
        })),
        worldsCopy: vi.fn(async () => 5),
        downloadsGet: vi.fn(async () => ({ folder: "C:\\dl", used: 100, files: [] })),
        downloadRemove: vi.fn(async () => {}),
        downloadsRemoveUnfinished: vi.fn(async () => 0),
        aboutGet: vi.fn(async () => about),
        openPath: vi.fn(async () => {}),
        openUrl: vi.fn(async () => {}),
        ...overrides,
    };
}

// the time the stores see; a test moves it
let now = 4 * 60_000;

function store(overrides: Partial<SettingsApi> = {}, navigate = vi.fn(), say = vi.fn(), copy = vi.fn(async () => {})) {
    return new SettingsStore(api(overrides), navigate, copy, say, () => now);
}

describe("SettingsStore", () => {
    it("sends an unknown section to registries", async () => {
        const settings = store();

        await settings.open("nonsense");
        expect(settings.section).toBe("registries");
        await settings.open(undefined);
        expect(settings.section).toBe("registries");
        await settings.open("downloads");
        expect(settings.section).toBe("downloads");
        expect(SettingsStore.sectionOf("about")).toBe("about");
    });

    it("keeps the add dialog open with the problem when the URL is refused", async () => {
        const refused = { kind: "operation-failed" as const, code: "https://x/ is not a registry", values: {}, restored: false };
        const registryAdd = vi.fn(async () => {
            throw new ApiFault(refused);
        });
        const settings = store({ registryAdd });
        await settings.open("registries");

        settings.openAdd();
        await settings.add("https://x/");

        expect(settings.addOpen).toBe(true);
        expect(settings.addProblem).toBe("That did not work.");
        settings.setAddValue("https://y/");
        expect(settings.addProblem).toBeUndefined();
    });

    it("closes the add dialog and lists the registries again when the URL is accepted", async () => {
        const registriesRefresh = vi.fn(async () => [registry("official", true)]);
        const settings = store({ registriesRefresh });
        await settings.open("registries");
        settings.openAdd();

        await settings.add("https://z/");

        expect(settings.addOpen).toBe(false);
        expect(registriesRefresh).toHaveBeenCalledTimes(2);
    });

    it("removes a registry by its row id and never the official one", async () => {
        const registryRemove = vi.fn(async () => {});
        const settings = store({ registryRemove });
        await settings.open("registries");

        await settings.removeRegistry("official");
        await settings.removeRegistry("friends");

        expect(registryRemove).toHaveBeenCalledExactlyOnceWith("https://friends/");
    });

    it("removes a version only after the confirmation", async () => {
        const versionRemove = vi.fn(async () => true);
        const settings = store({ versionRemove });
        await settings.open("versions");

        settings.askRemoveVersion("1.26.60.05");
        expect(versionRemove).not.toHaveBeenCalled();
        settings.cancelRemoveVersion();
        await settings.confirmRemoveVersion();
        expect(versionRemove).not.toHaveBeenCalled();

        settings.askRemoveVersion("1.26.60.05");
        await settings.confirmRemoveVersion();
        expect(versionRemove).toHaveBeenCalledExactlyOnceWith("1.26.60.05");
        expect(settings.pendingVersion).toBeNull();
    });

    it("counts the versions of both channels and shows the folder of the versions", async () => {
        now = 1_000;
        const settings = store();
        await settings.open("versions");
        now = 1_000 + 4 * 60_000;

        expect(settings.releaseCount).toBe(1);
        expect(settings.previewCount).toBe(1);
        expect(settings.versionsFetched()).toBe("4 min ago");
        expect(settings.versionsFolder()).toBe("%LOCALAPPDATA%\\amml\\versions");
        expect(settings.gameVersions().map((held) => held.state)).toEqual(["installed"]);
    });

    it("shows the installed versions when the lists cannot be read", async () => {
        const settings = store({
            versionsList: vi.fn(async () => {
                throw new ApiFault({ kind: "busy", code: null, values: {}, restored: false });
            }),
        });

        await settings.open("versions");

        expect(settings.gameVersions()).toHaveLength(1);
        expect(settings.releaseCount).toBe(0);
        expect(settings.versionsFetched()).toBe("never");
    });

    it("turns the preview switch through the bridge and reads the settings again", async () => {
        const settingsSetPreview = vi.fn(async () => {});
        const settingsGet = vi.fn(async () => ({ preview: true, dataDir: "d", logsDir: "C:\\l\\launcher\\logs" }));
        const settings = store({ settingsSetPreview, settingsGet });
        await settings.open("versions");

        await settings.setPreview(true);

        expect(settingsSetPreview).toHaveBeenCalledWith(true);
        expect(settings.settings?.preview).toBe(true);
    });

    it("copies the world sources job to the install screen", async () => {
        const worldsCopy = vi.fn(async () => 5);
        const navigate = vi.fn();
        const settings = store({ worldsCopy }, navigate);
        await settings.open("shared");

        await settings.copyWorlds("appdata");

        expect(worldsCopy).toHaveBeenCalledWith("appdata");
        expect(navigate).toHaveBeenCalledWith("/instances/install");
        expect(settings.worldSources()[0]?.name).toBe("Store Minecraft");
        expect(settings.stats()?.worlds).toBe(2);
    });

    it("opens web links with the system and folders as folders, and says the report was copied", async () => {
        const openUrl = vi.fn(async () => {});
        const openPath = vi.fn(async () => {});
        const say = vi.fn();
        const copy = vi.fn(async () => {});
        const settings = store({ openUrl, openPath }, vi.fn(), say, copy);
        await settings.open("about");

        await settings.openLink("https://github.com/Alaydriem");
        await settings.openLink(about.logsDir);
        await settings.copyReport("the report");

        expect(openUrl).toHaveBeenCalledExactlyOnceWith("https://github.com/Alaydriem");
        expect(openPath).toHaveBeenCalledExactlyOnceWith(about.logsDir);
        expect(copy).toHaveBeenCalledWith("the report");
        expect(say).toHaveBeenCalledWith("Report copied");
        expect(settings.platform()).toBe("Windows x64");
        const lines = settings.report();
        expect(lines.map((line) => line.id)).toEqual(["windows", "store", "devMode", "registry", "logs"]);
        expect(lines.find((line) => line.id === "windows")?.value).toBe("Windows 10.0.26300.7171");
        expect(lines.find((line) => line.id === "devMode")?.value).toBe("On");
        expect(lines.find((line) => line.id === "logs")?.href).toBe(about.logsDir);
    });
});
