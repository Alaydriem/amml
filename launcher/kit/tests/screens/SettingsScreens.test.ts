import { fireEvent, render, screen, within } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { DownloadsSettingsScreenRules } from "../../src/screens/DownloadsSettingsScreen";
import DownloadsSettingsScreen from "../../src/screens/DownloadsSettingsScreen.svelte";
import { RegistriesSettingsScreenRules } from "../../src/screens/RegistriesSettingsScreen";
import SharedDataSettingsScreen from "../../src/screens/SharedDataSettingsScreen.svelte";
import { SettingsFixtures } from "../../src/fixtures/settings";
import { downloadsSettingsEn } from "../../src/text/en/downloadsSettings";
import { registriesSettingsEn } from "../../src/text/en/registriesSettings";
import { sharedDataSettingsEn } from "../../src/text/en/sharedDataSettings";

const downloads = {
    folder: SettingsFixtures.downloadsFolder,
    usedBytes: SettingsFixtures.downloadsUsedBytes,
    limit: "10",
    files: SettingsFixtures.downloadFiles,
    text: downloadsSettingsEn,
};

describe("DownloadsSettingsScreen", () => {
    it("shows every file size in GB with two decimals", () => {
        expect(DownloadsSettingsScreenRules.fileSize(20_000_000)).toBe("0.02 GB");
        expect(DownloadsSettingsScreenRules.fileSize(2_040_000_000)).toBe("2.04 GB");

        render(DownloadsSettingsScreen, downloads);

        expect(screen.getByText("0.02 GB")).toBeInTheDocument();
        expect(screen.getByText("1.28 GB")).toBeInTheDocument();
    });

    it("shows the folder total as one figure with a small unit and no limit controls", () => {
        expect(DownloadsSettingsScreenRules.used(3_340_000_000)).toEqual({ value: "3.34", unit: "GB" });
        expect(DownloadsSettingsScreenRules.used(340_000_000)).toEqual({ value: "340", unit: "MB" });

        render(DownloadsSettingsScreen, downloads);

        expect(screen.getByText("3.34")).toBeInTheDocument();
        expect(screen.getByText("GB")).toBeInTheDocument();
        expect(screen.queryByRole("combobox")).toBeNull();
        expect(screen.queryByRole("progressbar")).toBeNull();
        expect(screen.queryByText(/limit/i)).toBeNull();
    });

    it("removes a file by its id and offers no Remove for a kept file", async () => {
        const onremovefile = vi.fn();
        render(DownloadsSettingsScreen, { ...downloads, onremovefile });

        expect(screen.getAllByRole("button", { name: "Remove" })).toHaveLength(2);
        const row = screen.getByText("zoom-26.60.5-windows-x64.zip").closest(".amml-panel-row") as HTMLElement;
        await fireEvent.click(within(row).getByRole("button", { name: "Remove" }));

        expect(onremovefile).toHaveBeenCalledExactlyOnceWith("zoom-zip");
    });

    it("offers Remove unfinished only while a file is unfinished", () => {
        const { unmount } = render(DownloadsSettingsScreen, downloads);
        expect(screen.getByRole("button", { name: "Remove unfinished" })).toBeInTheDocument();
        unmount();

        render(DownloadsSettingsScreen, {
            ...downloads,
            files: downloads.files.filter((file) => file.state !== "unfinished"),
        });
        expect(screen.queryByRole("button", { name: "Remove unfinished" })).toBeNull();
    });
});

describe("SharedDataSettingsScreen", () => {
    it("shows the size on disk with a space before the unit", () => {
        const { container } = render(SharedDataSettingsScreen, {
            stats: SettingsFixtures.sharedStats,
            locations: SettingsFixtures.worldLocations,
            sources: SettingsFixtures.worldSources,
            text: sharedDataSettingsEn,
        });

        const stat = Array.from(container.querySelectorAll(".amml-stat")).find((node) => node.textContent?.includes("On disk"));
        expect(stat?.querySelector(".amml-stat__value")?.textContent?.replace(/\s+/g, " ").trim()).toBe("3.2 GB");
    });

    it("copies worlds from a source by its id", async () => {
        const oncopyworlds = vi.fn();
        render(SharedDataSettingsScreen, {
            stats: SettingsFixtures.sharedStats,
            locations: SettingsFixtures.worldLocations,
            sources: SettingsFixtures.worldSources,
            text: sharedDataSettingsEn,
            oncopyworlds,
        });

        await fireEvent.click(screen.getAllByRole("button", { name: "Copy" })[0] as HTMLElement);

        expect(oncopyworlds).toHaveBeenCalledExactlyOnceWith("store");
    });
});

describe("RegistriesSettingsScreenRules", () => {
    const [official, friends] = SettingsFixtures.registries;

    it("reports the count and the fetch time of a registry that answered", () => {
        expect(RegistriesSettingsScreenRules.status(official!, registriesSettingsEn)).toEqual({
            text: "12 mods · 4 min ago",
            tone: "ok",
        });
        expect(RegistriesSettingsScreenRules.status({ ...official!, modCount: 1 }, registriesSettingsEn).text).toBe(
            "1 mod · 4 min ago",
        );
    });

    it("leaves the count out when it is unknown and marks a failed fetch as a fault", () => {
        expect(RegistriesSettingsScreenRules.status({ ...official!, modCount: undefined }, registriesSettingsEn).text).toBe(
            "4 min ago",
        );
        expect(RegistriesSettingsScreenRules.status(friends!, registriesSettingsEn)).toEqual({
            text: "Fetch failed",
            tone: "fault",
        });
    });
});
