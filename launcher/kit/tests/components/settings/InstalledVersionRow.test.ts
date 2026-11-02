import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { InstalledVersionRowRules } from "../../../src/components/settings/InstalledVersionRow";
import InstalledVersionRow from "../../../src/components/settings/InstalledVersionRow.svelte";
import { gameVersionsSettingsEn } from "../../../src/text/en/gameVersionsSettings";

const base = { version: "1.26.60.5", sizeText: "1.9 GB", text: gameVersionsSettingsEn.row };

describe("InstalledVersionRow", () => {
    it("allows removal only when no instance uses the version", () => {
        expect(InstalledVersionRowRules.canRemove(0)).toBe(true);
        expect(InstalledVersionRowRules.canRemove(1)).toBe(false);
    });

    it("disables Remove while an instance uses the version and does not call back", async () => {
        const onremove = vi.fn();
        render(InstalledVersionRow, { ...base, instanceCount: 2, onremove });

        const button = screen.getByRole("button", { name: "Remove" });
        expect(button).toBeDisabled();
        await fireEvent.click(button);

        expect(onremove).not.toHaveBeenCalled();
        expect(screen.getByText("2 instances")).toBeInTheDocument();
    });

    it("says one instance in the singular", () => {
        render(InstalledVersionRow, { ...base, instanceCount: 1 });

        expect(screen.getByText("1 instance")).toBeInTheDocument();
    });

    it("calls onremove with the version when no instance uses it", async () => {
        const onremove = vi.fn();
        render(InstalledVersionRow, { ...base, instanceCount: 0, onremove });

        const button = screen.getByRole("button", { name: "Remove" });
        expect(button).toBeEnabled();
        await fireEvent.click(button);

        expect(screen.getByText("No instances")).toBeInTheDocument();
        expect(onremove).toHaveBeenCalledExactlyOnceWith("1.26.60.5");
    });
});
