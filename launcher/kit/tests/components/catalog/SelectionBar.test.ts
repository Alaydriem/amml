import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SelectionBar from "../../../src/components/catalog/SelectionBar.svelte";
import { SelectionBarRules } from "../../../src/components/catalog/SelectionBar";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModBrowseEn } from "../../../src/text/en/addModBrowse";

const text = addModBrowseEn.bar;
const chunkBorders = CatalogFixtures.chunkBordersCard;
const clearWeather = CatalogFixtures.clearWeatherCard;

describe("SelectionBarRules", () => {
    it("sums the sizes and names the mods in order", () => {
        const cards = [chunkBorders, clearWeather];

        expect(SelectionBarRules.count(cards)).toBe(2);
        expect(SelectionBarRules.totalBytes(cards)).toBe(500_000);
        expect(SelectionBarRules.totalSize(cards)).toBe("0.5 MB");
        expect(SelectionBarRules.names(cards, ", ")).toBe("Chunk Borders, Clear Weather");
    });

    it("picks the singular, the plural and the empty install text", () => {
        expect(SelectionBarRules.installLabel([chunkBorders], text)).toBe("Install 1 mod");
        expect(SelectionBarRules.installLabel([chunkBorders, clearWeather], text)).toBe("Install 2 mods");
        expect(SelectionBarRules.installLabel([], text)).toBe("Install");
    });

    it("splits the summary into a strong count and the plain names", () => {
        expect(SelectionBarRules.summary([chunkBorders, clearWeather], text)).toEqual([
            { text: "2 selected", strong: true },
            { text: " · Chunk Borders, Clear Weather", strong: false },
        ]);
    });
});

describe("SelectionBar", () => {
    it("shows the count, the names, the total size and the install text", () => {
        render(SelectionBar, { selected: [chunkBorders, clearWeather], text });

        expect(screen.getByText("2 selected").tagName).toBe("STRONG");
        expect(screen.getByText(/Chunk Borders, Clear Weather/)).toBeInTheDocument();
        expect(screen.getByText("0.5 MB")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Install 2 mods" })).toBeEnabled();
    });

    it("shows the hint and a disabled Install button while nothing is ticked", async () => {
        const oninstall = vi.fn();
        render(SelectionBar, { selected: [], text, oninstall });

        expect(screen.getByText("Tick mods to install several at once.")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Clear" })).toBeNull();
        const install = screen.getByRole("button", { name: "Install" });
        expect(install).toBeDisabled();

        await fireEvent.click(install);
        expect(oninstall).not.toHaveBeenCalled();
    });

    it("calls onclear and oninstall", async () => {
        const onclear = vi.fn();
        const oninstall = vi.fn();
        render(SelectionBar, { selected: [chunkBorders], text, onclear, oninstall });

        await fireEvent.click(screen.getByRole("button", { name: "Clear" }));
        await fireEvent.click(screen.getByRole("button", { name: "Install 1 mod" }));

        expect(onclear).toHaveBeenCalledOnce();
        expect(oninstall).toHaveBeenCalledOnce();
    });
});
