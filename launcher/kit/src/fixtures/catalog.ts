import type {
    AddModContext,
    FoundMod,
    ModCard,
    ModDetail,
    ModDetailPending,
    RegistrySource,
    ResultGroup,
    Swatch,
    UrlCheck,
} from "../model";

type CardSeed = [id: string, title: string, summary: string, version: string, swatch: Swatch, size: number];

export class CatalogFixtures {
    /** The name of the instance the add-mod screens add to. */
    static readonly instanceName = "1.26.60.05";

    /** The chip on every add-mod screen. */
    static readonly context: AddModContext = { game: "1.26.60.5", amml: "26.60.5", api: "2" };

    static readonly registries: RegistrySource[] = [
        { id: "official", name: "Official", url: "mods.alaydriem.com", builtIn: true, modCount: 48, status: "ok" },
        { id: "community", name: "Community index", url: "index.example.org/amml", builtIn: false, modCount: 1601, status: "ok" },
        { id: "friends", name: "Friends' server pack", url: "pack.example.net/amml", builtIn: false, modCount: 212, status: "ok" },
        {
            id: "old",
            name: "Old test registry",
            url: "example.net/amml-test/registry.json",
            builtIn: false,
            status: "offline",
            fetched: "2 min ago",
        },
    ];

    static readonly allCount = 1861;

    private static card(seed: CardSeed, registryId: string, state: ModCard["state"] = "available"): ModCard {
        const [id, title, summary, version, swatch, size] = seed;
        return { id, registryId, title, summary, tags: [], version, size, swatch, state };
    }

    static readonly chunkBordersCard: ModCard = CatalogFixtures.card(["@kv/chunkborders", "Chunk Borders", "Draws chunk edges and marks slime chunks.", "1.1.2", 6, 300_000], "community");

    static readonly clearWeatherCard: ModCard = CatalogFixtures.card(["@tobi/clearweather", "Clear Weather", "Turns rain and fog off on your screen only.", "0.9.0", 8, 200_000], "community");

    static readonly clockHudCard: ModCard = CatalogFixtures.card(["@sunny/clockhud", "Clock HUD", "A small clock with the in-game time and moon phase.", "3.0.1", 9, 250_000], "community", "installed");

    /** The first loaded run of the community group: rows 41 to 60 of "C". */
    static readonly communityRows: ModCard[] = [
        CatalogFixtures.card(["@mira/chatheads", "Chat Heads", "Shows the speaker's face next to each chat line.", "2.4.0", 4, 400_000], "community"),
        CatalogFixtures.chunkBordersCard,
        CatalogFixtures.clearWeatherCard,
        CatalogFixtures.clockHudCard,
        CatalogFixtures.card(["@wayfarer/coordsplus", "Coords Plus", "Copy coordinates, save waypoints, share them in chat, and see the distance and direction to each one on the compass bar.", "1.7.0", 2, 520_000], "community"),
        CatalogFixtures.card(["@pixl/crosshair", "Crosshair Studio", "Draw your own crosshair, with colour that changes on targets.", "4.2.0", 10, 610_000], "community"),
        CatalogFixtures.card(["@nova/customsky", "Custom Sky", "Swap the sky box per dimension.", "2.0.0", 3, 480_000], "community", "update"),
        CatalogFixtures.card(["@logs/cuttree", "Cut Tree", "Break one log, the whole tree comes down.", "1.3.0", 7, 150_000], "community"),
        CatalogFixtures.card(["@zed/cyanfilter", "Cyan Filter", "Tints the screen for night builds.", "1.0.2", 5, 90_000], "community"),
        CatalogFixtures.card(["@zed/cycleclock", "Cycle Clock", "Shows the day and night cycle as a ring.", "0.4.0", 1, 110_000], "community"),
        CatalogFixtures.card(["@nova/cyclopseye", "Cyclops Eye", "A wide zoom that keeps the hand in view.", "2.2.1", 3, 330_000], "community"),
        CatalogFixtures.card(["@logs/cypresstrails", "Cypress Trails", "Leaves a fading line where you walked.", "1.5.0", 7, 140_000], "community"),
        CatalogFixtures.card(["@pixl/cyberhud", "Cyber HUD", "A sharper health and hunger display.", "3.1.0", 4, 270_000], "community"),
        CatalogFixtures.card(["@pixl/cybersky", "Cyber Sky", "Neon sky colours for every dimension.", "1.2.0", 2, 360_000], "community"),
        CatalogFixtures.card(["@sunny/cyclestats", "Cycle Stats", "Counts days, nights and storms.", "0.8.3", 9, 120_000], "community"),
        CatalogFixtures.card(["@mira/cygnuschat", "Cygnus Chat", "Chat tabs with their own filters.", "2.0.4", 6, 450_000], "community"),
        CatalogFixtures.card(["@kv/cylinder", "Cylinder Guide", "Draw a circle of blocks around any point.", "1.0.0", 10, 200_000], "community"),
        CatalogFixtures.card(["@tobi/cymbal", "Cymbal", "Sounds for each block you place.", "0.2.1", 8, 80_000], "community"),
        CatalogFixtures.card(["@wayfarer/cypherkeys", "Cypher Keys", "Rebind every key from one screen.", "1.4.2", 2, 160_000], "community"),
        CatalogFixtures.card(["@nova/cyprusnights", "Cyprus Nights", "Warmer light after dusk.", "1.1.0", 9, 130_000], "community"),
    ];

    /** Artboard 06b: rows 41 to 60 loaded, the rest of the 1,412 are missing, 20 rows to a page. */
    static browseGroups(): ResultGroup[] {
        return [
            {
                id: "c",
                label: "C",
                registryId: "community",
                rows: CatalogFixtures.communityRows,
                offset: 40,
                total: 1412,
                pageSize: 20,
            },
        ];
    }

    /**
     * The group with the rows of registry index page `page` loaded, as the registry returns them. The rows between
     * the loaded range and the page are loaded too, so the range stays in one piece.
     */
    static withPage(group: ResultGroup, page: number): ResultGroup {
        const start = group.start ?? 0;
        const pageStart = Math.max((page - 1) * group.pageSize, start);
        const pageEnd = Math.min(page * group.pageSize, start + group.total);
        const loadedEnd = group.offset + group.rows.length;
        if (pageStart >= loadedEnd) {
            return { ...group, rows: [...group.rows, ...CatalogFixtures.generated(group.registryId, loadedEnd, pageEnd)] };
        }
        if (pageStart >= group.offset) return group;
        return {
            ...group,
            offset: pageStart,
            rows: [...CatalogFixtures.generated(group.registryId, pageStart, group.offset), ...group.rows],
        };
    }

    /** Placeholder rows `from` to `to`, not including `to`. */
    static generated(registryId: string, from: number, to: number): ModCard[] {
        const rows: ModCard[] = [];
        for (let i = from; i < to; i++) {
            rows.push({
                id: `@sample/mod-${i + 1}`,
                registryId,
                title: `Sample Mod ${i + 1}`,
                summary: "A placeholder row that stands in for a mod loaded from the registry.",
                tags: [],
                version: `1.${i % 10}.0`,
                size: 200_000,
                swatch: ((i % 10) + 1) as Swatch,
                state: "available",
            });
        }
        return rows;
    }

    static readonly mapperCard: ModCard = CatalogFixtures.card(["@carto/mapper", "Mapper", "A full-screen world map that fills in as you explore.", "3.2.0", 4, 1_100_000], "community");

    static readonly mapArtCard: ModCard = CatalogFixtures.card(["@pixl/mapart", "Map Art Helper", "Shows which block to place next for map art.", "1.0.4", 8, 300_000], "community");

    /** Artboard 06c: the query "map" across three registries; the community group stops at four rows. */
    static searchGroups(): ResultGroup[] {
        return [
            {
                id: "official",
                label: "Official",
                registryId: "official",
                rows: [CatalogFixtures.card(["@alaydriem/minimap", "Minimap", "A small map in the corner of the screen.", "26.60.1", 4, 700_000], "official", "installed")],
                offset: 0,
                total: 1,
                pageSize: 20,
            },
            {
                id: "community",
                label: "Community index",
                registryId: "community",
                rows: [
                    CatalogFixtures.mapperCard,
                    CatalogFixtures.mapArtCard,
                    CatalogFixtures.card(["@kv/minimapplus", "Mini Map Plus", "Another corner map, with mob dots.", "0.8.1", 9, 420_000], "community"),
                    CatalogFixtures.card(["@wayfarer/markers", "Map Markers", "Pin places on in-game maps.", "1.2.0", 2, 260_000], "community"),
                ],
                offset: 0,
                total: 27,
                pageSize: 20,
                collapsedMore: 23,
            },
            {
                id: "friends",
                label: "Friends' server pack",
                registryId: "friends",
                rows: [
                    CatalogFixtures.card(["@friends/servermap", "Server Map", "Our server's spawn, shops and farms.", "1.0.0", 10, 180_000], "friends"),
                    CatalogFixtures.card(["@friends/mappins", "Map Pins", "Share map pins with the group.", "0.3.0", 3, 90_000], "friends"),
                    CatalogFixtures.card(["@friends/maptour", "Map Tour", "Walk the group through saved places.", "0.1.0", 7, 70_000], "friends"),
                ],
                offset: 0,
                total: 3,
                pageSize: 20,
            },
        ];
    }

    static readonly chunkBorders: ModDetail = {
        ...CatalogFixtures.chunkBordersCard,
        registryName: "Community index",
        description: "Draws chunk edges and marks slime chunks. Toggle with a key; colours follow your settings.",
        authors: ["kv"],
        license: "MIT",
        sourceLabel: "kv/chunkborders",
        sourceUrl: "https://github.com/kv/chunkborders",
        api: 2,
        hasChecksum: true,
    };

    static readonly mapper: ModDetail = {
        ...CatalogFixtures.mapperCard,
        registryName: "Community index",
        description: "A full-screen world map that fills in as you explore. Waypoints and death points.",
        authors: ["carto"],
        license: "MIT",
        sourceLabel: "carto/mapper",
        sourceUrl: "https://github.com/carto/mapper",
        api: 2,
        hasChecksum: true,
    };

    /** Artboard 06d. */
    static readonly urlChecks: UrlCheck[] = [
        { id: "zip", title: "Downloaded the zip", severity: "ok", value: "0.6 MB" },
        { id: "folder", title: "Holds one mod, and its amml.hcl matches the folder", severity: "ok", value: "@someone/nightvision", valueTone: "dim" },
        { id: "api", title: "Works with this instance's AMML", severity: "ok", value: "api 2", valueTone: "ok" },
        { id: "registry", title: "Not in any of your registries", severity: "warn", value: "no checksum" },
    ];

    static readonly url =
        "https://github.com/someone/nightvision/releases/download/1.2.0/nightvision-1.2.0-windows-x64.zip";

    static readonly found: FoundMod = { label: "Night Vision Plus 1.2.0", id: "@someone/nightvision" };

    /** The mono line under the search field on artboard 06b. */
    static readonly browseLine = "1,412 work with api 2 · 189 hidden · showing 41–60";

    /** The mono line on artboard 06c. */
    static readonly searchLine = "31 results in 3 registries · Old test registry is offline";

    /** Artboard 06f: the line, the query and the mods the other registries hold for it. */
    static readonly noResultsLine = "0 results in Community index · 4 in other registries";

    static readonly noResultsQuery = "dynmap";

    static readonly noResultsOtherMatches = 4;

    /** Artboard 09: the mod document of Mapper is still loading. */
    static readonly pendingMapper: ModDetailPending = {
        card: CatalogFixtures.mapperCard,
        registryName: "Community index",
        api: 2,
        document: "mods/@carto/mapper.json",
    };
}
