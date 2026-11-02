import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import ModResultRow from "../../../src/components/catalog/ModResultRow.svelte";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModBrowseEn } from "../../../src/text/en/addModBrowse";

const text = { installed: "Installed", update: "Update", select: addModBrowseEn.list.selectRow };
const card = CatalogFixtures.chunkBordersCard;

describe("ModResultRow", () => {
    it("opens the mod from the keyboard through its main button", async () => {
        const user = userEvent.setup();
        const onactivate = vi.fn();
        render(ModResultRow, { card, text, onactivate });

        screen.getByRole("button", { name: /Chunk Borders/ }).focus();
        await user.keyboard("{Enter}");
        await user.keyboard(" ");

        expect(onactivate).toHaveBeenCalledTimes(2);
        expect(onactivate).toHaveBeenCalledWith("@kv/chunkborders");
    });

    it("marks the active row as the current one", () => {
        render(ModResultRow, { card, text, active: true });

        expect(screen.getByRole("button", { name: /Chunk Borders/ })).toHaveAttribute("aria-current", "true");
    });

    it("keeps the tick and the open action apart", async () => {
        const user = userEvent.setup();
        const onactivate = vi.fn();
        const onchange = vi.fn();
        render(ModResultRow, { card, text, onactivate, onchange });

        await user.click(screen.getByRole("checkbox", { name: "Select Chunk Borders" }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith("@kv/chunkborders", true);
        expect(onactivate).not.toHaveBeenCalled();
    });
});
