import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { AddRegistryDialogRules } from "../../../src/components/dialogs/AddRegistryDialog";
import AddRegistryDialog from "../../../src/components/dialogs/AddRegistryDialog.svelte";
import { dialogsEn } from "../../../src/text/en/dialogs";

const props = { open: true, text: dialogsEn.addRegistry };

describe("AddRegistryDialogRules.canAdd", () => {
    it("needs an https address and no problem from the app", () => {
        expect(AddRegistryDialogRules.canAdd("http://x/")).toBe(false);
        expect(AddRegistryDialogRules.canAdd("")).toBe(false);
        expect(AddRegistryDialogRules.canAdd("  https://x/ ")).toBe(true);
        expect(AddRegistryDialogRules.canAdd("https://x/", "refused")).toBe(false);
    });
});

describe("AddRegistryDialog", () => {
    it("disables Add for an address that is not https", () => {
        const { unmount } = render(AddRegistryDialog, { ...props, value: "http://x/" });
        expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
        expect(screen.getByText("Use an https:// address.")).toBeInTheDocument();
        unmount();

        const empty = render(AddRegistryDialog, { ...props, value: "" });
        expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
        expect(screen.queryByText("Use an https:// address.")).toBeNull();
        empty.unmount();

        render(AddRegistryDialog, { ...props, value: "https://x/" });
        expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
    });

    it("calls onadd with the trimmed address", async () => {
        const onadd = vi.fn();
        render(AddRegistryDialog, { ...props, value: "  https://x/  ", onadd });

        await fireEvent.click(screen.getByRole("button", { name: "Add" }));

        expect(onadd).toHaveBeenCalledExactlyOnceWith("https://x/");
    });

    it("shows the problem and disables Add while it is set", async () => {
        const onadd = vi.fn();
        render(AddRegistryDialog, { ...props, value: "https://x/", problem: "That registry has no index.", onadd });

        expect(screen.getByText("That registry has no index.")).toBeInTheDocument();
        const add = screen.getByRole("button", { name: "Add" });
        expect(add).toBeDisabled();
        await fireEvent.click(add);

        expect(onadd).not.toHaveBeenCalled();
    });
});
