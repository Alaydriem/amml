import type { GameVersion, Instance, NewInstanceSummary } from "../model";

/** Content of artboards 03 and 04. */
export class InstanceFixtures {
    static readonly instances: Instance[] = [
        {
            id: "1.26.60.05",
            name: "1.26.60.05",
            swatch: 3,
            gameVersion: "1.26.60.5",
            ammlVersion: "26.60.5",
            modCount: 7,
            dataMode: "shared",
            folder: "%LOCALAPPDATA%\\amml\\instances\\1.26.60.05",
            activity: "last played 2 h ago",
            running: false,
            updateCount: 2,
        },
        {
            id: "1.26.60.05-scenes",
            name: "1.26.60.05-scenes",
            swatch: 5,
            gameVersion: "1.26.60.5",
            ammlVersion: "26.60.5",
            modCount: 5,
            dataMode: "shared",
            folder: "%LOCALAPPDATA%\\amml\\instances\\1.26.60.05-scenes",
            activity: "running since 14:02",
            running: true,
            updateCount: 1,
        },
        {
            id: "1.26.52.3-prores",
            name: "1.26.52.3-prores",
            swatch: 9,
            gameVersion: "1.26.52.3",
            ammlVersion: "26.52.1",
            modCount: 4,
            dataMode: "own",
            folder: "%LOCALAPPDATA%\\amml\\instances\\1.26.52.3-prores",
            activity: "last played 3 d ago",
            running: false,
            updateCount: 0,
        },
    ];

    static readonly versions: GameVersion[] = [
        { version: "1.26.60.5", channel: "release", state: "installed", size: 3_900_000_000, instanceCount: 2 },
        { version: "1.26.52.3", channel: "release", state: "installed", size: 3_700_000_000, instanceCount: 1 },
        { version: "1.26.51.1", channel: "release", state: "download", size: 4_100_000_000, instanceCount: 0 },
        { version: "1.26.50.2", channel: "release", state: "unsupported", instanceCount: 0 },
        { version: "1.26.41.1", channel: "release", state: "unsupported", instanceCount: 0 },
        { version: "1.26.70.20", channel: "preview", state: "download", size: 4_300_000_000, instanceCount: 0 },
        { version: "1.26.70.10", channel: "preview", state: "unsupported", instanceCount: 0 },
    ];

    static readonly summary: NewInstanceSummary = {
        ammlVersion: "26.60.5",
        defaultMods: 6,
        diskBytes: 1_900_000_000,
    };

    /** Name Paper shows in the new instance form. */
    static readonly draftName = "1.26.60.05-players";

    /** Version Paper shows selected in the new instance list. */
    static readonly selectedVersion = "1.26.60.5";
}
