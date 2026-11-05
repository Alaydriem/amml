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

    it("mod_result_row_needs_amml_is_greyed_without_install", async () => {
        const user = userEvent.setup();
        const onactivate = vi.fn();
        const onchange = vi.fn();
        const blocked = { ...card, state: "needsAmml" as const, blockedNote: "Needs AMML 26.62.0 (game 1.26.70.1)" };
        const { container } = render(ModResultRow, { card: blocked, text, onactivate, onchange });

        expect(screen.getByText("Needs AMML 26.62.0 (game 1.26.70.1)")).toBeInTheDocument();
        expect(container.querySelector(".amml-result--blocked")).not.toBeNull();
        expect(screen.getByRole("checkbox", { name: "Select Chunk Borders" })).toBeDisabled();
        expect(screen.queryByText("Update")).toBeNull();

        await user.click(screen.getByRole("button", { name: /Chunk Borders/ }));
        expect(onactivate).toHaveBeenCalledWith("@kv/chunkborders");
    });

    it("shows the later-release note on an installed card and on an available card that stays tickable", async () => {
        const user = userEvent.setup();
        const onchange = vi.fn();
        const note = "Later release needs AMML 26.62.0";
        const installed = render(ModResultRow, { card: { ...card, state: "installed", laterNote: note }, text });
        expect(screen.getByText(note)).toBeInTheDocument();
        expect(installed.container.querySelector(".amml-result--blocked")).toBeNull();
        installed.unmount();

        const available = render(ModResultRow, { card: { ...card, state: "available", laterNote: note }, text, onchange });
        expect(screen.getByText(note)).toBeInTheDocument();
        expect(available.container.querySelector(".amml-result--blocked")).toBeNull();
        await user.click(screen.getByRole("checkbox", { name: "Select Chunk Borders" }));
        expect(onchange).toHaveBeenCalledExactlyOnceWith("@kv/chunkborders", true);
    });
});
