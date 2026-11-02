import { describe, expect, it, test } from "vitest";
import literal from "./fixtures/Literal.svelte.txt?raw";
import { DesignLiteralScan } from "./DesignLiteralScan";

const CLEAN_AREAS = ["components/primitives", "components/shell", "components/dialogs", "components/settings", "components/setup", "components/instances", "components/install", "components/mods", "components/catalog"];

describe("design literals", () => {
    describe.each(CLEAN_AREAS)("%s", (folder) => {
        it("has no colour, font size, letter spacing, font weight, radius or opacity written as a literal", () => {
            expect(DesignLiteralScan.scanFolder(folder)).toEqual([]);
        });

        it("uses only tokens that tokens.css or the file itself defines", () => {
            const tokens = DesignLiteralScan.scanFolder(folder).filter((issue) => issue.kind === "undefined token");
            expect(tokens).toEqual([]);
        });
    });

    describe("setup, instance and install screens", () => {
        const SCREENS = [
            "screens/FirstRunChecksScreen.svelte",
            "screens/FirstRunAdoptScreen.svelte",
            "screens/InstancesScreen.svelte",
            "screens/NewInstanceScreen.svelte",
            "screens/InstallProgressScreen.svelte",
        ];

        it("have no design literal and use only defined tokens", () => {
            const issues = DesignLiteralScan.scanFolder("screens").filter((issue) => SCREENS.includes(issue.file));
            expect(issues).toEqual([]);
        });
    });

    describe("mod list and catalog screens", () => {
        const SCREENS = [
            "screens/InstanceDetailScreen.svelte",
            "screens/AddModBrowseScreen.svelte",
            "screens/AddModUrlScreen.svelte",
        ];

        it("have no design literal and use only defined tokens", () => {
            const issues = DesignLiteralScan.scanFolder("screens").filter((issue) => SCREENS.includes(issue.file));
            expect(issues).toEqual([]);
        });
    });

    it("finds a literal that is not a token", () => {
        const issues = DesignLiteralScan.scanSource("Literal.svelte", literal);
        expect(issues.map((issue) => issue.kind).sort()).toEqual([
            "font size",
            "font weight",
            "hex colour",
            "letter spacing",
            "named colour",
            "opacity",
            "radius",
            "undefined token",
        ]);
    });
});
