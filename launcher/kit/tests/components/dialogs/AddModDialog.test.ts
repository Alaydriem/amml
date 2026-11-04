import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { AddModDialogRules } from "../../../src/components/dialogs/AddModDialog";
import AddModDialog from "../../../src/components/dialogs/AddModDialog.svelte";
import { dialogsEn } from "../../../src/text/en/dialogs";

const props = {
    open: true,
    text: dialogsEn.addMod,
    values: { name: "1.26.60.05" },
    knownIds: ["@alaydriem/peek"],
    suggestions: [{ id: "@alaydriem/peek", title: "Peek" }],
};

describe("AddModDialog", () => {
    it("treats an input no registry lists as unofficial", () => {
        expect(AddModDialogRules.isUnofficial("https://example.org/a.zip", props.knownIds)).toBe(true);
        expect(AddModDialogRules.isUnofficial("@alaydriem/peek", props.knownIds)).toBe(false);
        expect(AddModDialogRules.isUnofficial("   ", props.knownIds)).toBe(false);
    });

    it("keeps Install unofficial disabled for an empty input and shows no warning", () => {
        render(AddModDialog, { ...props, value: "" });

        expect(screen.getByRole("button", { name: "Install unofficial" })).toBeDisabled();
        expect(screen.queryByText("Not in any registry.")).toBeNull();
    });

    it("keeps Install unofficial disabled for an id a registry lists", async () => {
        const oninstallurl = vi.fn();
        render(AddModDialog, { ...props, value: "@alaydriem/peek", oninstallurl });

        const button = screen.getByRole("button", { name: "Install unofficial" });
        expect(button).toBeDisabled();
        await fireEvent.click(button);

        expect(oninstallurl).not.toHaveBeenCalled();
    });

    it("warns, with the lead sentence in bold, and installs the trimmed URL for an unknown input", async () => {
        const oninstallurl = vi.fn();
        render(AddModDialog, { ...props, value: "  https://example.org/a.zip ", oninstallurl });

        expect(screen.getByText("Not in any registry.").tagName).toBe("STRONG");
        await fireEvent.click(screen.getByRole("button", { name: "Install unofficial" }));

        expect(oninstallurl).toHaveBeenCalledExactlyOnceWith("https://example.org/a.zip");
    });

    it("installs the trimmed URL when Enter is pressed in the field", async () => {
        const oninstallurl = vi.fn();
        render(AddModDialog, { ...props, value: " https://example.org/a.zip", oninstallurl });

        await fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });

        expect(oninstallurl).toHaveBeenCalledExactlyOnceWith("https://example.org/a.zip");
    });

    it("names the field from its visible caption and fills the title slot", () => {
        render(AddModDialog, { ...props, value: "" });

        expect(screen.getByRole("textbox", { name: "Registry id or zip URL" })).toBeInTheDocument();
        expect(screen.getByRole("dialog", { name: "Add a mod to 1.26.60.05" })).toBeInTheDocument();
    });

    it("asks the parent for the typed text and keeps showing the prop", async () => {
        const oninput = vi.fn();
        render(AddModDialog, { ...props, value: "abc", oninput });

        const field = screen.getByRole("textbox");
        await fireEvent.input(field, { target: { value: "abcd" } });

        expect(oninput).toHaveBeenCalledExactlyOnceWith("abcd");
        expect(field).toHaveValue("abc");
    });

    it("calls onadd with the id of a suggestion", async () => {
        const onadd = vi.fn();
        render(AddModDialog, { ...props, value: "", onadd });

        await fireEvent.click(screen.getByRole("button", { name: "Add" }));

        expect(onadd).toHaveBeenCalledExactlyOnceWith("@alaydriem/peek");
    });
});
