import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import UrlInstallPane from "../../../src/components/catalog/UrlInstallPane.svelte";
import { UrlInstallPaneRules } from "../../../src/components/catalog/UrlInstallPane";
import { CatalogFixtures } from "../../../src/fixtures/catalog";
import { addModUrlEn } from "../../../src/text/en/addModUrl";

const text = addModUrlEn.pane;
const link = "https://example.org/mod.zip";

describe("UrlInstallPaneRules.canCheck", () => {
    it("needs a link and no check running", () => {
        expect(UrlInstallPaneRules.canCheck(link, false)).toBe(true);
        expect(UrlInstallPaneRules.canCheck("   ", false)).toBe(false);
        expect(UrlInstallPaneRules.canCheck(link, true)).toBe(false);
    });
});

describe("UrlInstallPane", () => {
    it("checks the link when Enter is pressed in the field", async () => {
        const oncheck = vi.fn();
        render(UrlInstallPane, { url: link, checks: [], text, oncheck });

        await fireEvent.keyDown(screen.getByRole("textbox", { name: text.urlLabel }), { key: "Enter" });

        expect(oncheck).toHaveBeenCalledExactlyOnceWith(link);
    });

    it("ignores Enter in an empty field and while a check runs", async () => {
        const oncheck = vi.fn();
        const { rerender } = render(UrlInstallPane, { url: "", checks: [], text, oncheck });
        await fireEvent.keyDown(screen.getByRole("textbox", { name: text.urlLabel }), { key: "Enter" });

        await rerender({ url: link, checking: true });
        await fireEvent.keyDown(screen.getByRole("textbox", { name: text.urlLabel }), { key: "Enter" });

        expect(oncheck).not.toHaveBeenCalled();
        expect(screen.getByRole("button", { name: "Check" })).toBeDisabled();
    });

    it("asks for the typed link and keeps the shown one until the parent answers", async () => {
        const oninput = vi.fn();
        render(UrlInstallPane, { url: "", checks: [], text, oninput });
        const field = screen.getByRole("textbox", { name: text.urlLabel });

        await fireEvent.input(field, { target: { value: "https://x.example/a.zip" } });

        expect(oninput).toHaveBeenCalledExactlyOnceWith("https://x.example/a.zip");
        expect(field).toHaveValue("");
    });

    it("lists the checks and shows the unofficial callout only when asked", async () => {
        const { rerender } = render(UrlInstallPane, { url: link, checks: CatalogFixtures.urlChecks, text });
        expect(screen.getByText("Downloaded the zip")).toBeInTheDocument();
        expect(screen.queryByText("This installs as unofficial.")).toBeNull();

        await rerender({ unofficial: true });

        expect(screen.getByText("This installs as unofficial.").tagName).toBe("STRONG");
    });
});
