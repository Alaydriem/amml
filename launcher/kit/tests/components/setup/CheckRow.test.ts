import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import CheckRow from "../../../src/components/setup/CheckRow.svelte";
import { CheckRowRules } from "../../../src/components/setup/CheckRow";
import { firstRunChecksEn } from "../../../src/text/en/firstRunChecks";

const text = firstRunChecksEn.check;

describe("CheckRow", () => {
    it("calls onaction with the check id from the action button", async () => {
        const onaction = vi.fn();
        render(CheckRow, {
            check: { id: "gameinput", title: "GameInput", severity: "warn", actionLabel: "Install", elevated: true },
            text,
            onaction,
        });
        await fireEvent.click(screen.getByRole("button", { name: "Install" }));
        expect(onaction).toHaveBeenCalledExactlyOnceWith("gameinput");
    });

    it("shows the value and no button when there is no action", () => {
        render(CheckRow, { check: { id: "store", title: "Store", severity: "ok", value: "Found" }, text });
        expect(screen.getByText("Found")).toBeInTheDocument();
        expect(screen.queryByRole("button")).toBeNull();
    });

    it.each([
        ["ok", "Passed"],
        ["warn", "Needs attention"],
        ["muted", "Optional"],
    ] as const)("names a %s check for assistive technology as %s", (severity, name) => {
        render(CheckRow, { check: { id: "c", title: "Check", severity }, text });
        expect(screen.getByText(name)).toHaveClass("amml-visually-hidden");
    });
});

describe("CheckRowRules", () => {
    it("draws numbers and paths in mono and words in sans", () => {
        expect(CheckRowRules.isMonoValue("212 GB free")).toBe(true);
        expect(CheckRowRules.isMonoValue("api 2")).toBe(true);
        expect(CheckRowRules.isMonoValue("Found")).toBe(false);
        expect(CheckRowRules.isMonoValue(undefined)).toBe(false);
    });

    it("maps muted to the empty box", () => {
        expect(CheckRowRules.mark("muted")).toBe("pending");
        expect(CheckRowRules.mark("warn")).toBe("warn");
    });
});
