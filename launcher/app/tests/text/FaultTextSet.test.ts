import { afterEach, describe, expect, it } from "vitest";
import { I18n } from "#lib/i18n/index.ts";
import { CheckTextSet } from "#lib/text/CheckTextSet.ts";
import { FaultTextSet } from "#lib/text/FaultTextSet.ts";

describe("FaultTextSet", () => {
    afterEach(() => I18n.adopt(null));

    it("fills the disk-space fault with sizes in the kit's units", () => {
        const text = FaultTextSet.build("disk-space", { drive: "C:\\", needed: "6100000000", free: "3400000000" });

        expect(text.title).toBe("Not enough space on C:\\.");
        expect(text.body).toBe(
            "This install needs 6.1 GB at once for the download, the copy and the instance. 3.4 GB is free.",
        );
        expect(text.action).toBe("Check again");
    });

    it("names the figures of a failed download, a failed copy and a refused install", () => {
        expect(FaultTextSet.build("download-failed", { done: "1280000000", total: "2060000000" }).body).toBe(
            "1.3 GB of 2.1 GB is kept and the download continues from there.",
        );
        expect(FaultTextSet.build("copy-failed", { copied: "9412", expected: "12178" }).title).toBe(
            "Copy failed at 9,412 of 12,178 files.",
        );
        expect(FaultTextSet.build("deploy-refused", { version: "1.26.51.1" }).title).toBe(
            "Windows refused to install 1.26.51.1.",
        );
        expect(FaultTextSet.build("install-interrupted", { version: "1.26.60.5" }).body).toBe(
            "The launcher finished putting your Store version 1.26.60.5 back. It starts as before.",
        );
    });

    it("keeps its promise of a download without figures when the bridge has none", () => {
        const body = FaultTextSet.build("download-failed", {}).body;
        expect(body).toBe("What came down is kept and the download continues from there.");
    });

    it("says the Store version was not put back, and never claims nothing changed", () => {
        const restore = FaultTextSet.build("restore-failed", {});
        expect(restore.title).toBe("Your Store version was not put back.");
        expect(restore.action).toBe("Copy details");
        const general = FaultTextSet.build("operation-failed", {});
        expect(general.body).toBe("The details are below.");
        expect(general.body).not.toContain("Nothing was changed");
    });

    it("words the faults the app raises itself", () => {
        expect(FaultTextSet.build("busy", {}).action).toBe("Show progress");
        expect(FaultTextSet.build("instance-running", { name: "1.26.60.05" }).title).toBe("1.26.60.05 is running.");
        expect(FaultTextSet.build("consent-needed", {})).toEqual({ title: "", body: "", action: "" });
    });
});

describe("CheckTextSet", () => {
    afterEach(() => I18n.adopt(null));

    const found = {
        id: "store" as const,
        state: "ok" as const,
        package: "Microsoft.MinecraftUWP_1.26.6005.0_x64__8wekyb3d8bbwe",
        version: "1.26.60.5",
        freeBytes: null,
        drive: null,
        action: null,
        elevated: false,
    };

    it("words a found Store game, a low disk and a missing GameInput", () => {
        expect(CheckTextSet.detail(found)).toBe("1.26.60.5 · Microsoft.MinecraftUWP");
        expect(CheckTextSet.value(found)).toBe("Found");

        const disk = { ...found, id: "disk" as const, package: null, version: null, freeBytes: 212_000_000_000, drive: "C:\\" };
        expect(CheckTextSet.detail(disk)).toBe("C:\\ · 6 GB per new version");
        expect(CheckTextSet.value(disk)).toBe("212 GB free");

        const input = { ...found, id: "gameinput" as const, state: "missing" as const, action: "install" as const, elevated: true };
        expect(CheckTextSet.value(input)).toBeUndefined();
        expect(CheckTextSet.action(input)).toBe("Install");
        expect(CheckTextSet.title("gameinput")).toBe("GameInput");
    });

    it("offers the Store for a missing Store game and says a missing preview app is not installed", () => {
        const missing = { ...found, state: "missing" as const, package: null, version: null, action: "open-store" as const };
        expect(CheckTextSet.detail(missing)).toBe("Install Minecraft Bedrock from the Microsoft Store");
        expect(CheckTextSet.value(missing)).toBe("Not found");
        expect(CheckTextSet.action(missing)).toBe("Open the Store");
        expect(CheckTextSet.value({ ...missing, id: "preview" })).toBe("Not installed");
    });
});
