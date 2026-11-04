import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ContextChip from "../../../src/components/primitives/ContextChip.svelte";
import KeyValueRow from "../../../src/components/primitives/KeyValueRow.svelte";
import RichText from "../../../src/components/primitives/RichText.svelte";
import StatusMark from "../../../src/components/primitives/StatusMark.svelte";
import { TileRules } from "../../../src/components/primitives/Tile";
import { Template } from "../../../src/text/Template";

describe("RichText", () => {
    it("draws the strong runs in strong elements and the rest as plain text", () => {
        const { container } = render(RichText, { parts: Template.parts("**3 updates** in {where}", { where: "2 instances" }) });

        expect(container.querySelector("strong")).toHaveTextContent("3 updates");
        expect(container).toHaveTextContent("3 updates in 2 instances");
    });
});

describe("StatusMark", () => {
    it("says its state to assistive technology without drawing the words", () => {
        render(StatusMark, { state: "failed", label: "Failed" });

        const words = screen.getByText("Failed");
        expect(words).toHaveClass("amml-visually-hidden");
    });

    it("is decorative without a label", () => {
        const { container } = render(StatusMark, { state: "ok" });

        expect(container.querySelector(".amml-visually-hidden")).toBeNull();
    });
});

describe("KeyValueRow", () => {
    it("makes the value a button when onopen is set", async () => {
        const onopen = vi.fn();
        render(KeyValueRow, { label: "Logs", value: "launcher\\logs", onopen });

        await fireEvent.click(screen.getByRole("button", { name: "launcher\\logs" }));

        expect(onopen).toHaveBeenCalledOnce();
    });

    it("draws a plain value without onopen", () => {
        render(KeyValueRow, { label: "Windows", value: "11 Pro" });

        expect(screen.queryByRole("button")).toBeNull();
    });
});

describe("ContextChip", () => {
    it("puts the separator between pairs and not before the first", () => {
        const { container } = render(ContextChip, {
            parts: [
                { id: "game", label: "Game", value: "1.26.60.5" },
                { id: "api", label: "api", value: "2", tone: "ok" },
            ],
            text: { separator: "·" },
        });

        expect(container.querySelectorAll(".amml-chip__separator")).toHaveLength(1);
    });
});

describe("TileRules.swatchColor", () => {
    it("pads the swatch number to the spectrum token", () => {
        expect(TileRules.swatchColor(3)).toBe("var(--color-spectrum-03)");
        expect(TileRules.swatchColor(10)).toBe("var(--color-spectrum-10)");
    });
});
