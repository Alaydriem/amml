import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ModTable from "../../../src/components/mods/ModTable.svelte";
import { ModTableRules } from "../../../src/components/mods/ModTable";
import { InstanceDetailScreenRules } from "../../../src/screens/InstanceDetailScreen";
import { ModFixtures } from "../../../src/fixtures/mods";
import { instanceDetailEn } from "../../../src/text/en/instanceDetail";

const mods = ModFixtures.installed;
const text = InstanceDetailScreenRules.tableText(instanceDetailEn, mods);

describe("ModTableRules.updateCount", () => {
    it("counts the rows whose state is update", () => {
        expect(ModTableRules.updateCount(mods)).toBe(2);
    });

    it("does not count a mod from a URL, even when a newer version is known", () => {
        const url = { ...ModFixtures.nightVision, availableVersion: "9.9.9" };

        expect(ModTableRules.updateCount([url, ModFixtures.zoom])).toBe(0);
    });

    it("does not count a version it cannot read", () => {
        expect(ModTableRules.updateCount([{ ...ModFixtures.zoom, availableVersion: "latest" }])).toBe(0);
    });
});

describe("ModTable", () => {
    it("names the update count on the Update all button and calls onupdateall", async () => {
        const onupdateall = vi.fn();
        render(ModTable, { mods, text, menuItems: instanceDetailEn.menu, onupdateall });

        await fireEvent.click(screen.getByRole("button", { name: "Update all (2)" }));

        expect(onupdateall).toHaveBeenCalledOnce();
    });

    it("disables Update all when only a URL mod has a newer version", () => {
        const only = [{ ...ModFixtures.nightVision, availableVersion: "9.9.9" }];
        render(ModTable, {
            mods: only,
            text: InstanceDetailScreenRules.tableText(instanceDetailEn, only),
            menuItems: instanceDetailEn.menu,
        });

        expect(screen.getByRole("button", { name: "Update all (0)" })).toBeDisabled();
    });

    it("gives every column head a role inside a row group", () => {
        render(ModTable, { mods, text, menuItems: instanceDetailEn.menu });

        const heads = screen.getAllByRole("columnheader");
        expect(heads.map((head) => head.textContent?.trim())).toEqual([
            "On",
            "Name",
            "Installed",
            "Available",
            "Status",
            "Actions",
        ]);
        expect(screen.getAllByRole("rowgroup")).toHaveLength(2);
        expect(screen.getByRole("table", { name: "Mods in this instance" })).toBeInTheDocument();
    });
});
