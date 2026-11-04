import type { InstanceDetailScreenText } from "../../screens/InstanceDetailScreen";

export const instanceDetailEn: InstanceDetailScreenText = {
    sectionLabel: "Core · {core} · Mods · {mods}",
    updateAll: "Update all ({count})",
    addMod: "Add a mod",
    columns: {
        on: "On",
        name: "Name",
        installed: "Installed",
        available: "Available",
        state: "Status",
        actions: "Actions",
    },
    tableLabel: "Mods in this instance",
    row: {
        coreTag: "Core",
        upToDate: "Up to date",
        update: "Update",
        unofficial: "Unofficial",
        noVersion: "—",
        toggleLabel: "Turn {title} on or off",
        menuLabel: "Actions for {title}",
        subline: "{id} · {note}",
    },
    menu: [
        { id: "folder", label: "Show in folder" },
        { id: "remove", label: "Remove", tone: "fault" },
    ],
};
