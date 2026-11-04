import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Checkbox from "../../../src/components/primitives/Checkbox.svelte";

describe("Checkbox", () => {
    it("asks for the opposite state and calls back when enabled", async () => {
        const onchange = vi.fn();
        render(Checkbox, { label: "Chunk Borders", checked: false, onchange });

        await fireEvent.click(screen.getByRole("checkbox", { name: "Chunk Borders" }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith(true);
    });

    it("keeps showing the prop when the parent refuses the change", async () => {
        const onchange = vi.fn();
        render(Checkbox, { label: "Chunk Borders", checked: true, onchange });
        const box = screen.getByRole("checkbox", { name: "Chunk Borders" });

        await fireEvent.click(box);
        await fireEvent.click(box);

        expect(onchange).toHaveBeenNthCalledWith(1, false);
        expect(onchange).toHaveBeenNthCalledWith(2, false);
        expect(box).toHaveAttribute("aria-checked", "true");
    });

    it("does not call back when disabled", async () => {
        const onchange = vi.fn();
        render(Checkbox, { label: "Chunk Borders", checked: true, disabled: true, onchange });
        const box = screen.getByRole("checkbox", { name: "Chunk Borders" });

        await fireEvent.click(box);

        expect(box).toHaveAttribute("aria-checked", "true");
        expect(onchange).not.toHaveBeenCalled();
    });

    it("draws a decorative mark that is not a control", () => {
        render(Checkbox, { decorative: true, checked: true, variant: "marker" });

        expect(screen.queryByRole("checkbox")).toBeNull();
        expect(document.querySelector(".amml-checkbox")).toHaveAttribute("aria-hidden", "true");
    });
});
