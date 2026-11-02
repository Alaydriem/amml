import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import Segmented from "../../../src/components/primitives/Segmented.svelte";

const options = [
    { id: "release", label: "Release" },
    { id: "preview", label: "Preview" },
    { id: "beta", label: "Beta" },
];

describe("Segmented", () => {
    it("asks for the next and the previous option with the arrow keys", async () => {
        const onchange = vi.fn();
        render(Segmented, { options, value: "preview", label: "Channel", onchange });

        await fireEvent.keyDown(screen.getByRole("radio", { name: "Preview" }), { key: "ArrowRight" });
        expect(onchange).toHaveBeenLastCalledWith("beta");

        await fireEvent.keyDown(screen.getByRole("radio", { name: "Preview" }), { key: "ArrowLeft" });
        expect(onchange).toHaveBeenLastCalledWith("release");
    });

    it("keeps showing the prop when the parent refuses the change", async () => {
        const onchange = vi.fn();
        render(Segmented, { options, value: "release", label: "Channel", onchange });

        await fireEvent.click(screen.getByRole("radio", { name: "Beta" }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith("beta");
        expect(screen.getByRole("radio", { name: "Release" })).toHaveAttribute("aria-checked", "true");
        expect(screen.getByRole("radio", { name: "Beta" })).toHaveAttribute("aria-checked", "false");
    });

    it("wraps from the last option to the first", async () => {
        const onchange = vi.fn();
        render(Segmented, { options, value: "beta", label: "Channel", onchange });

        await fireEvent.keyDown(screen.getByRole("radio", { name: "Beta" }), { key: "ArrowRight" });

        expect(onchange).toHaveBeenLastCalledWith("release");
    });

    it("keeps one tab stop on the selected option", () => {
        render(Segmented, { options, value: "beta", label: "Channel" });

        expect(screen.getByRole("radio", { name: "Beta" })).toHaveAttribute("tabindex", "0");
        expect(screen.getByRole("radio", { name: "Release" })).toHaveAttribute("tabindex", "-1");
    });

    it("ignores keys when disabled", async () => {
        const onchange = vi.fn();
        render(Segmented, { options, value: "release", label: "Channel", disabled: true, onchange });

        await fireEvent.keyDown(screen.getByRole("radio", { name: "Release" }), { key: "ArrowRight" });

        expect(onchange).not.toHaveBeenCalled();
    });
});
