import type { AdoptedInstance, ChoiceItem, SystemCheck } from "../model";

/** Content of artboards 01 and 02. */
export class SetupFixtures {
    static readonly checks: SystemCheck[] = [
        {
            id: "store",
            title: "Minecraft from the Store",
            detail: "1.26.60.5 · Microsoft.MinecraftUWP",
            severity: "ok",
            value: "Found",
        },
        {
            id: "gaming-services",
            title: "Gaming Services",
            detail: "Needed to install game versions",
            severity: "ok",
            value: "Found",
        },
        {
            id: "gameinput",
            title: "GameInput",
            detail: "Controllers and some mods need it",
            severity: "warn",
            actionLabel: "Install",
            elevated: true,
        },
        {
            id: "preview",
            title: "Minecraft Preview",
            detail: "Only for preview builds",
            severity: "muted",
            value: "Not installed",
        },
        {
            id: "disk",
            title: "Disk space",
            detail: "C:\\ · 6 GB per new version",
            severity: "ok",
            value: "212 GB free",
        },
    ];

    static readonly adopted: AdoptedInstance[] = [
        { id: "a", name: "1.26.60.05", swatch: 3, modCount: 7 },
        { id: "b", name: "1.26.60.05-scenes", swatch: 5, ammlVersion: "26.60.5", modCount: 5 },
    ];

    static readonly choices: ChoiceItem[] = [
        {
            id: "instance",
            title: "1.26.60.05 instance",
            detail: "instances\\1.26.60.05\\Minecraft Bedrock",
            meta: "14 worlds · today",
        },
        {
            id: "store",
            title: "Store Minecraft",
            detail: "%APPDATA%\\Minecraft Bedrock",
            meta: "3 worlds · 5 mo ago",
        },
        {
            id: "scratch",
            title: "Start from scratch",
            detail: "AMML will create a clean Minecraft Bedrock instance",
        },
    ];

    /** Id of the choice Paper shows selected. */
    static readonly selectedChoice = "instance";
}
