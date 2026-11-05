import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ModTableRow from "../../../src/components/mods/ModTableRow.svelte";
import { ModTableRowRules } from "../../../src/components/mods/ModTableRow";
import { ModFixtures } from "../../../src/fixtures/mods";
import { instanceDetailEn } from "../../../src/text/en/instanceDetail";

const text = instanceDetailEn.row;
const menuItems = instanceDetailEn.menu;
const { coreLoader, freecam, modManager, nightVision, zoom } = ModFixtures;

describe("ModTableRowRules", () => {
    it("gives a core mod the tag and neither toggle nor menu", () => {
        expect(ModTableRowRules.controls(coreLoader)).toEqual({ toggle: false, menu: false, coreTag: true });
        expect(ModTableRowRules.controls(zoom)).toEqual({ toggle: true, menu: true, coreTag: false });
    });

    it("finds an update only when the available version is newer", () => {
        expect(ModTableRowRules.hasUpdate(modManager)).toBe(true);
        expect(ModTableRowRules.hasUpdate(zoom)).toBe(false);
        expect(ModTableRowRules.hasUpdate({ ...zoom, installedVersion: "26.60.9" })).toBe(false);
        expect(ModTableRowRules.hasUpdate(nightVision)).toBe(false);
    });

    it.each([
        ["26.60.10", "26.60.9", true],
        ["26.60.9", "26.60.10", false],
        ["26.60", "26.60.0", false],
        ["26.60.1", "26.60", true],
        ["1.0.0", "1.0.0", false],
        ["v1.2.0", "1.1.9", true],
        ["1.2.0", "1.2.0-beta", true],
        ["1.2.0-beta", "1.2.0", false],
        ["1.0.0-rc1", "1.1.0", false],
        ["1.1.0-rc1", "1.0.0", true],
        ["1.2.0-beta.2", "1.2.0-beta.10", false],
        ["1.2.0-beta.10", "1.2.0-beta.2", true],
        ["1.2.0-rc", "1.2.0-beta", true],
        ["1.2.0-beta", "1.2.0-beta.1", false],
        ["1.2.0+build5", "1.2.0", false],
        ["beta-2", "beta-1", false],
        ["26.60.5", "unknown", false],
        ["", "1.0.0", false],
    ])("compares %s with %s as newer: %s", (available, installed, newer) => {
        expect(ModTableRowRules.isNewer(available, installed)).toBe(newer);
    });

    it("marks a URL mod unofficial before it looks for an update", () => {
        expect(ModTableRowRules.state(nightVision)).toBe("unofficial");
        expect(ModTableRowRules.state({ ...nightVision, availableVersion: "9.9.9" })).toBe("unofficial");
        expect(ModTableRowRules.state(freecam)).toBe("update");
        expect(ModTableRowRules.state(zoom)).toBe("current");
    });

    it("joins the id and the note, or shows the id alone", () => {
        expect(ModTableRowRules.subline(zoom, text)).toBe("@alaydriem/zoom · official");
        expect(ModTableRowRules.subline({ ...zoom, note: undefined }, text)).toBe("@alaydriem/zoom");
    });
});

describe("ModTableRow", () => {
    it("shows the Core tag and no toggle and no menu for a core mod", () => {
        render(ModTableRow, { mod: coreLoader, text, menuItems });

        expect(screen.getByText("Core")).toBeInTheDocument();
        expect(screen.queryByRole("switch")).toBeNull();
        expect(screen.queryByRole("button", { name: /Actions for/ })).toBeNull();
    });

    it("calls ontoggle with the id and the new state and keeps the shown state until the parent answers", async () => {
        const ontoggle = vi.fn();
        render(ModTableRow, { mod: zoom, text, menuItems, ontoggle });
        const toggle = screen.getByRole("switch", { name: "Turn Zoom on or off" });

        await fireEvent.click(toggle);

        expect(ontoggle).toHaveBeenCalledExactlyOnceWith("@alaydriem/zoom", false);
        expect(toggle).toHaveAttribute("aria-checked", "true");
    });

    it("calls onmenu with the mod id and the item id", async () => {
        const onmenu = vi.fn();
        render(ModTableRow, { mod: zoom, text, menuItems, onmenu });

        await fireEvent.click(screen.getByRole("button", { name: "Actions for Zoom" }));
        await fireEvent.click(screen.getByRole("menuitem", { name: "Remove" }));

        expect(onmenu).toHaveBeenCalledExactlyOnceWith("@alaydriem/zoom", "remove");
    });

    it("shows Update for a newer version and calls onupdate", async () => {
        const onupdate = vi.fn();
        render(ModTableRow, { mod: freecam, text, menuItems, onupdate });

        await fireEvent.click(screen.getByRole("button", { name: "Update" }));

        expect(onupdate).toHaveBeenCalledExactlyOnceWith("@alaydriem/freecam");
    });

    it("shows the unofficial pill and the dash for a URL mod", () => {
        render(ModTableRow, { mod: nightVision, text, menuItems });

        expect(screen.getByText("Unofficial")).toBeInTheDocument();
        expect(screen.getByText("—")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Update" })).toBeNull();
    });
});

describe("ModTableRow local build", () => {
    const local = { ...coreLoader, installedVersion: "26.60.5+local", availableVersion: "26.60.5", localBuild: true };

    it("local_build_with_equal_base_version_shows_update_pill", () => {
        expect(ModTableRowRules.state(local)).toBe("update");
        render(ModTableRow, { mod: local, text, menuItems });
        expect(screen.getByText(text.localBuild)).toBeTruthy();
        expect(screen.getByRole("button", { name: text.update })).toBeTruthy();
    });

    it("local_build_with_newer_release_shows_update_pill", () => {
        const newer = { ...local, availableVersion: "26.61.0" };
        expect(ModTableRowRules.state(newer)).toBe("update");
    });

    it("local_build_offline_shows_badge_only", () => {
        const offline = { ...local, availableVersion: undefined };
        expect(ModTableRowRules.state(offline)).toBe("current");
        render(ModTableRow, { mod: offline, text, menuItems });
        expect(screen.getByText(text.localBuild)).toBeTruthy();
        expect(screen.queryByRole("button", { name: text.update })).toBeNull();
    });

    it("shows no local badge for a release", () => {
        render(ModTableRow, { mod: coreLoader, text, menuItems });
        expect(screen.queryByText(text.localBuild)).toBeNull();
    });
});

describe("ModTableRow blocked note", () => {
    it("mod_table_row_blocked_note_hides_update", () => {
        const blocked = { ...modManager, blockedNote: "Needs AMML 26.62.0" };
        const { container } = render(ModTableRow, { mod: blocked, text, menuItems });

        expect(screen.getByText("Needs AMML 26.62.0")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: text.update })).toBeNull();
        expect(container.querySelector(".amml-mod-row--blocked")).not.toBeNull();
        expect(screen.queryByText(text.upToDate)).toBeNull();
    });

    it("keeps the update action next to a later-release note", () => {
        render(ModTableRow, { mod: { ...modManager, laterNote: "Later release needs AMML 26.62.0" }, text, menuItems });

        expect(screen.getByText("Later release needs AMML 26.62.0")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: text.update })).toBeInTheDocument();
    });
});
