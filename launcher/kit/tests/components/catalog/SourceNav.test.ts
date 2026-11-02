import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SourceNav from "../../../src/components/catalog/SourceNav.svelte";
import { SourceNavRules } from "../../../src/components/catalog/SourceNav";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModBrowseEn } from "../../../src/text/en/addModBrowse";

const text = addModBrowseEn.nav;
const registries = CatalogFixtures.registries;

describe("SourceNavRules.trailing", () => {
    it("shows the count, or the state of a registry with no list", () => {
        expect(SourceNavRules.trailing(registries[1]!, text)).toEqual({ text: "1,601", fault: false });
        expect(SourceNavRules.trailing(registries[3]!, text)).toEqual({ text: "offline", fault: true });
        expect(SourceNavRules.trailing({ ...registries[3]!, status: "failed" }, text)).toEqual({ text: "failed", fault: true });
    });
});

describe("SourceNav", () => {
    it("marks the chosen source current and asks for another on a press", async () => {
        const onchange = vi.fn();
        render(SourceNav, { registries, allCount: 1861, value: "community", text, onchange });

        expect(screen.getByRole("button", { name: /Community index/ })).toHaveAttribute("aria-current", "true");
        expect(screen.getByRole("button", { name: /All registries/ })).not.toHaveAttribute("aria-current");

        await fireEvent.click(screen.getByRole("button", { name: /From a URL/ }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith(SourceNavRules.URL);
    });

    it("names the toggle by its visible caption and keeps its state until the parent answers", async () => {
        const onshowotheramml = vi.fn();
        render(SourceNav, { registries, text, onshowotheramml });
        const toggle = screen.getByRole("switch", { name: text.needOtherAmml });

        await fireEvent.click(toggle);

        expect(toggle).toHaveAttribute("aria-labelledby");
        expect(onshowotheramml).toHaveBeenCalledExactlyOnceWith(true);
        expect(toggle).toHaveAttribute("aria-checked", "false");
    });
});
