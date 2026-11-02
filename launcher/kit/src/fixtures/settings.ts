import type {
    AboutAuthor,
    AboutLink,
    BugReportLine,
    DownloadFile,
    GameVersion,
    Option,
    RegistrySource,
    SharedDataStats,
    WorldLocation,
    WorldSource,
} from "../model";

/** Data of artboards 07a to 07e, as Paper shows it. */
export class SettingsFixtures {
    static readonly registries: RegistrySource[] = [
        {
            id: "official",
            name: "Official",
            url: "alaydriem.github.io/amml-registry/registry.json",
            builtIn: true,
            modCount: 12,
            status: "ok",
            fetched: "4 min ago",
        },
        {
            id: "friends",
            name: "Friends' server pack",
            url: "example.org/amml/registry.json",
            builtIn: false,
            status: "failed",
        },
    ];

    static readonly versionSource = "MinecraftBedrockArchiver/GdkLinks · urls.json";
    static readonly releaseCount = 212;
    static readonly previewCount = 340;
    static readonly versionsFetched = "4 min ago";
    static readonly versionsFolder = "%LOCALAPPDATA%\\amml\\versions";

    static readonly versions: GameVersion[] = [
        { version: "1.26.60.5", channel: "release", state: "installed", size: 1_900_000_000, instanceCount: 2 },
        { version: "1.26.52.3", channel: "release", state: "installed", size: 1_900_000_000, instanceCount: 1 },
        { version: "1.26.51.1", channel: "release", state: "installed", size: 1_900_000_000, instanceCount: 0 },
    ];

    static readonly sharedStats: SharedDataStats = {
        folder: "%LOCALAPPDATA%\\amml\\data\\Minecraft Bedrock",
        worlds: 14,
        resourcePacks: 6,
        behaviorPacks: 3,
        bytes: 3_200_000_000,
    };

    static readonly worldLocations: WorldLocation[] = [
        { instanceId: "i1", name: "1.26.60.05", swatch: 3, mode: "shared" },
        { instanceId: "i2", name: "1.26.60.05-scenes", swatch: 5, mode: "shared" },
        {
            instanceId: "i3",
            name: "1.26.52.3-prores",
            swatch: 9,
            mode: "own",
            folder: "instances\\1.26.52.3-prores\\Minecraft Bedrock",
        },
    ];

    static readonly worldSources: WorldSource[] = [
        { id: "store", name: "Store Minecraft", folder: "%APPDATA%\\Minecraft Bedrock", worldCount: 3, modified: "5 mo ago" },
        { id: "i3", name: "1.26.52.3-prores", folder: "Own folder", worldCount: 2, modified: "3 d ago" },
    ];

    static readonly downloadsFolder = "%LOCALAPPDATA%\\amml\\launcher\\downloads";
    static readonly downloadsUsedBytes = 3_340_000_000;
    static readonly downloadsLimitBytes = 10_000_000_000;
    static readonly downloadsLimit = "10";
    static readonly downloadLimitOptions: Option[] = [
        { id: "5", label: "5 GB" },
        { id: "10", label: "10 GB" },
        { id: "20", label: "20 GB" },
        { id: "50", label: "50 GB" },
    ];

    static readonly downloadFiles: DownloadFile[] = [
        {
            id: "store-version",
            name: "1.26.60.5.msixvc",
            size: 2_040_000_000,
            state: "kept",
            detail: "Your Store version · kept for the restore step",
        },
        {
            id: "unfinished-version",
            name: "1.26.51.1.msixvc",
            size: 1_280_000_000,
            state: "unfinished",
            detail: "Unfinished · 62% · resumes on the next install",
        },
        {
            id: "zoom-zip",
            name: "zoom-26.60.5-windows-x64.zip",
            size: 20_000_000,
            state: "waiting",
            detail: "Waiting to install · checksum OK",
        },
    ];

    static readonly aboutVersion = "0.1.0";
    static readonly aboutBuiltOn = "2026-11-02";
    static readonly aboutPlatform = "Windows x64";
    static readonly aboutUpdateVersion = "0.1.1";

    static readonly aboutAuthor: AboutAuthor = {
        name: "Alaydriem",
        initials: "CP",
        link: "github.com/Alaydriem",
        url: "https://github.com/Alaydriem",
    };

    static readonly aboutLinks: AboutLink[] = [
        { id: "source", href: "https://github.com/Alaydriem/amml" },
        { id: "issues", href: "https://github.com/Alaydriem/amml/issues" },
        { id: "licence", href: "https://github.com/Alaydriem/amml/blob/master/LICENSE" },
        { id: "notices", href: "https://github.com/Alaydriem/amml/blob/master/NOTICE" },
    ];

    static readonly bugReport: BugReportLine[] = [
        { id: "windows", value: "11 Pro 26300 x64" },
        { id: "store", value: "1.26.60.5 · Status Ok" },
        { id: "devMode", value: "Off" },
        { id: "registry", value: "2026.11.02-3 · schema 2" },
        { id: "logs", value: "launcher\\logs", href: "launcher\\logs" },
    ];
}
