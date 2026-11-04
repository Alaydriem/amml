import type { NewInstanceScreenText } from "../../screens/NewInstanceScreen";

export const newInstanceEn: NewInstanceScreenText = {
    back: "Instances",
    title: "Create a new **instance**",
    versionLabel: "Game version",
    release: "Release",
    preview: "Preview",
    versionsLabel: "Game versions",
    versionRow: {
        installedNote: "Already in the version store · local copy only",
        installed: "Installed",
        download: "~{size} download",
        unsupported: "Not supported",
    },
    form: {
        nameLabel: "Instance name",
        folderHint: "Folder: {folder}",
        worldsLabel: "Worlds and packs",
        shared: "Shared",
        own: "Own folder",
        cancel: "Cancel",
        create: "Create",
    },
    folderPath: "instances\\{name}",
    installs: "Installs **{amml}** and **{mods}**.",
    amml: "AMML {version}",
    mods: { one: "{count} default mod", other: "{count} default mods" },
    disk: "About {size} on disk.",
};
