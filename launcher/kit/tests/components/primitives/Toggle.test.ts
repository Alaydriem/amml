import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Toggle from "../../../src/components/primitives/Toggle.svelte";

describe("Toggle", () => {
    it("asks for the opposite state and calls back when enabled", async () => {
        const onchange = vi.fn();
        render(Toggle, { label: "Preview", checked: false, onchange });

        await fireEvent.click(screen.getByRole("switch", { name: "Preview" }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith(true);
    });

    it("keeps showing the prop when the parent refuses the change", async () => {
        const onchange = vi.fn();
        render(Toggle, { label: "Preview", checked: false, onchange });
        const toggle = screen.getByRole("switch", { name: "Preview" });

        await fireEvent.click(toggle);

        expect(toggle).toHaveAttribute("aria-checked", "false");
    });

    it("does not call back when disabled", async () => {
        const onchange = vi.fn();
        render(Toggle, { label: "Preview", checked: false, disabled: true, onchange });

        await fireEvent.click(screen.getByRole("switch", { name: "Preview" }));

        expect(onchange).not.toHaveBeenCalled();
    });

    it("is named by a visible caption through labelledby", () => {
        const caption = document.createElement("span");
        caption.id = "caption";
        caption.textContent = "Show other AMML";
        document.body.append(caption);
        render(Toggle, { label: "fallback", labelledby: "caption", id: "t" });

        expect(screen.getByRole("switch", { name: "Show other AMML" })).toHaveAttribute("id", "t");
        caption.remove();
    });
});
