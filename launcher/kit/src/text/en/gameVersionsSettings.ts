import type { GameVersionsSettingsScreenText } from "../../screens/GameVersionsSettingsScreen";

export const gameVersionsSettingsEn: GameVersionsSettingsScreenText = {
    buildsHeading: "Builds",
    previewTitle: "Show preview builds",
    previewNote: "Needs Minecraft Preview from the Store. Few mods support them.",
    previewToggleLabel: "Show preview builds",
    listTitle: "Version list",
    listDetail: "{source} · {release} release, {preview} preview",
    refresh: "Refresh",
    installedHeading: "Installed versions · {size}",
    row: {
        instances: { one: "{count} instance", other: "{count} instances" },
        noInstances: "No instances",
        remove: "Remove",
    },
    note: "An installed version is the clean copy every instance of it is made from. A version in use by an instance cannot be removed. Removing one never touches your Store install.",
};
