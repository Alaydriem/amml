import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ConfirmDialog from "../../../src/components/dialogs/ConfirmDialog.svelte";
import { dialogsEn } from "../../../src/text/en/dialogs";

const base = { open: true, text: dialogsEn.deleteInstance, values: { name: "players" } };

describe("ConfirmDialog", () => {
    it("calls onconfirm from the confirm button only", async () => {
        const onconfirm = vi.fn();
        const oncancel = vi.fn();
        render(ConfirmDialog, { ...base, confirmVariant: "danger", onconfirm, oncancel });

        await fireEvent.click(screen.getByRole("button", { name: "Delete folder" }));

        expect(onconfirm).toHaveBeenCalledOnce();
        expect(oncancel).not.toHaveBeenCalled();
    });

    it("calls oncancel from the cancel button", async () => {
        const onconfirm = vi.fn();
        const oncancel = vi.fn();
        render(ConfirmDialog, { ...base, onconfirm, oncancel });

        await fireEvent.click(screen.getByRole("button", { name: "Keep it" }));

        expect(oncancel).toHaveBeenCalledOnce();
        expect(onconfirm).not.toHaveBeenCalled();
    });

    it("cancels on Escape", async () => {
        const onconfirm = vi.fn();
        const oncancel = vi.fn();
        render(ConfirmDialog, { ...base, onconfirm, oncancel });

        await fireEvent.keyDown(document.body, { key: "Escape" });

        expect(oncancel).toHaveBeenCalledOnce();
        expect(onconfirm).not.toHaveBeenCalled();
    });

    it("sends Escape to the layer on top only", async () => {
        const below = vi.fn();
        const above = vi.fn();
        render(ConfirmDialog, { ...base, oncancel: below });
        render(ConfirmDialog, {
            ...base,
            text: dialogsEn.developerMode,
            values: { file: "a.exe", version: "1" },
            oncancel: above,
        });

        await fireEvent.keyDown(document.body, { key: "Escape" });

        expect(above).toHaveBeenCalledOnce();
        expect(below).not.toHaveBeenCalled();
    });

    it("fills the title, draws the bold part of the body and shows the path and the note", () => {
        render(ConfirmDialog, {
            open: true,
            text: dialogsEn.developerMode,
            values: { file: "Minecraft.Windows.exe", version: "1.26.51.1" },
            path: "%LOCALAPPDATA%\\amml\\instances\\x",
        });

        expect(screen.getByRole("dialog", { name: "Turn on Developer Mode?" })).toBeInTheDocument();
        expect(screen.getByText("Minecraft.Windows.exe").tagName).toBe("STRONG");
        expect(screen.getByText("%LOCALAPPDATA%\\amml\\instances\\x")).toBeInTheDocument();
        expect(screen.getByText(dialogsEn.developerMode.note ?? "")).toBeInTheDocument();
    });
});
