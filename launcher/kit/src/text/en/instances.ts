import type { InstanceHeaderText } from "../../components/instances/InstanceHeader";
import type { InstanceRowText } from "../../components/instances/InstanceRow";
import type { MenuItem } from "../../components/primitives/Menu";
import type { InstancesScreenText } from "../../screens/InstancesScreen";

export const instanceRowEn: InstanceRowText = {
    game: "Game",
    amml: "AMML",
    mods: "Mods",
    launch: "Launch",
    running: "Running",
    upToDate: "Up to date",
    updates: { one: "{count} update", other: "{count} updates" },
    activityLine: "{data} · {activity}",
    dataShared: "Shared data",
    dataOwn: "Own data folder",
    menuLabel: "More actions",
};

export const instanceMenuEn: MenuItem[] = [
    { id: "open-folder", label: "Open folder" },
    { id: "delete", label: "Delete", tone: "fault" },
];

export const instanceHeaderEn: InstanceHeaderText = {
    back: "Instances",
    delete: "Delete",
    openFolder: "Open folder",
    launch: "Launch",
    running: "Running",
    detail: "Game {game} · {data} · {folder}",
    dataShared: "shared data",
    dataOwn: "own data folder",
};

export const instancesEn: InstancesScreenText = {
    eyebrow: "Instances · {count}",
    title: "Launch **AMML.**",
    newInstance: "New instance",
    banner: {
        message: "**{updates} available** in {instances}. Nothing is installed until you ask.",
        updates: { one: "{count} update", other: "{count} updates" },
        instances: { one: "{count} instance", other: "{count} instances" },
        review: "Review",
    },
    listLabel: "Instances",
    empty: {
        title: "No instances yet.",
        body: "An instance is one game version with AMML and its mods, in a folder of its own. Make as many as you like. Your Store install is never changed.",
    },
    row: instanceRowEn,
};
