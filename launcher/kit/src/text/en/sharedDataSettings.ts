import type { SharedDataSettingsScreenText } from "../../screens/SharedDataSettingsScreen";

export const sharedDataSettingsEn: SharedDataSettingsScreenText = {
    folder: {
        heading: "Shared folder",
        note: "Every instance set to Shared reads and writes here.",
        open: "Open",
        stats: {
            worlds: "Worlds",
            resourcePacks: "Resource packs",
            behaviorPacks: "Behavior packs",
            onDisk: "On disk",
        },
    },
    locationsHeading: "Where each instance keeps its worlds",
    location: {
        shared: "Shared",
        own: "Own folder",
    },
    copyInHeading: "Copy worlds in",
    source: {
        copy: "Copy",
    },
    worlds: { one: "{count} world", other: "{count} worlds" },
    sourceDetail: "{folder} · {worlds} · {modified}",
    note: "Copying never moves or deletes the source, and never overwrites a world already in the shared folder.",
};
