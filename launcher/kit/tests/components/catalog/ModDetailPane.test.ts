import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import ModDetailPane from "../../../src/components/catalog/ModDetailPane.svelte";
import { ModDetailPaneRules } from "../../../src/components/catalog/ModDetailPane";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModBrowseEn } from "../../../src/text/en/addModBrowse";

const text = addModBrowseEn.detail;

describe("ModDetailPaneRules", () => {
    it("joins the size and the checksum state", () => {
        expect(ModDetailPaneRules.downloadValue(CatalogFixtures.chunkBorders, text)).toBe("0.3 MB · checksum");
        expect(ModDetailPaneRules.downloadValue({ ...CatalogFixtures.chunkBorders, hasChecksum: false }, text)).toBe(
            "0.3 MB · no checksum",
        );
    });
});

describe("ModDetailPane", () => {
    it("asks for a mod while none is selected", () => {
        render(ModDetailPane, { text });

        expect(screen.getByText(text.empty)).toBeInTheDocument();
    });

    it("shows the mod document", () => {
        render(ModDetailPane, { detail: CatalogFixtures.chunkBorders, text });

        expect(screen.getByRole("heading", { name: "Chunk Borders" })).toBeInTheDocument();
        expect(screen.getByText("@kv/chunkborders · Community index")).toBeInTheDocument();
        expect(screen.getByText("0.3 MB · checksum")).toBeInTheDocument();
    });

    it("shows what the index row knows while the document loads, and marks the pane busy", () => {
        const { container } = render(ModDetailPane, { pending: CatalogFixtures.pendingMapper, text });

        expect(screen.getByRole("heading", { name: "Mapper" })).toBeInTheDocument();
        expect(screen.getByText("3.2.0")).toBeInTheDocument();
        expect(screen.getByText("loading mods/@carto/mapper.json")).toBeInTheDocument();
        expect(screen.queryByText(text.empty)).toBeNull();
        expect(container.querySelector("aside")).toHaveAttribute("aria-busy", "true");
    });

    it("prefers the loading state over an older document", () => {
        render(ModDetailPane, { detail: CatalogFixtures.chunkBorders, pending: CatalogFixtures.pendingMapper, text });

        expect(screen.getByRole("heading", { name: "Mapper" })).toBeInTheDocument();
        expect(screen.queryByText("Chunk Borders")).toBeNull();
    });
});
