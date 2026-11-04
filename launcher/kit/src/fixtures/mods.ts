import type { InstalledMod } from "../model";

/** The six mods of artboard 06. */
export class ModFixtures {
    static readonly coreLoader: InstalledMod = {
        id: "alaydriem/amml",
        title: "AMML loader",
        origin: "core",
        enabled: true,
        installedVersion: "26.60.5",
        availableVersion: "26.60.5",
        note: "api 2",
    };

    static readonly modManager: InstalledMod = {
        id: "@amml/modmanager",
        title: "Mod Manager",
        origin: "core",
        enabled: true,
        installedVersion: "26.60.2",
        availableVersion: "26.60.5",
        note: "core, always on",
    };

    static readonly zoom: InstalledMod = {
        id: "@alaydriem/zoom",
        title: "Zoom",
        origin: "registry",
        registryName: "Official",
        enabled: true,
        installedVersion: "26.60.5",
        availableVersion: "26.60.5",
        note: "official",
    };

    static readonly minimap: InstalledMod = {
        id: "@alaydriem/minimap",
        title: "Minimap",
        origin: "registry",
        registryName: "Official",
        enabled: false,
        installedVersion: "26.60.1",
        availableVersion: "26.60.1",
        note: "official",
    };

    static readonly freecam: InstalledMod = {
        id: "@alaydriem/freecam",
        title: "Freecam",
        origin: "registry",
        registryName: "Official",
        enabled: true,
        installedVersion: "26.60.0",
        availableVersion: "26.60.3",
        note: "official",
    };

    static readonly nightVision: InstalledMod = {
        id: "@someone/nightvision",
        title: "Night Vision Plus",
        origin: "url",
        enabled: true,
        installedVersion: "1.2.0",
        note: "from URL",
    };

    static readonly installed: InstalledMod[] = [
        ModFixtures.coreLoader,
        ModFixtures.modManager,
        ModFixtures.zoom,
        ModFixtures.minimap,
        ModFixtures.freecam,
        ModFixtures.nightVision,
    ];
}
