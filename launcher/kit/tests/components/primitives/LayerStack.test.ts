import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import { LayerStack } from "../../../src/components/primitives/LayerStack";
import LayerHost from "./LayerHost.svelte";

describe("LayerStack", () => {
    it("hands a key to the top layer only", () => {
        const lower = vi.fn();
        const upper = vi.fn();
        const first = LayerStack.open(lower);
        const second = LayerStack.open(upper);

        window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
        expect(upper).toHaveBeenCalledOnce();
        expect(lower).not.toHaveBeenCalled();

        LayerStack.close(second);
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
        expect(lower).toHaveBeenCalledOnce();

        LayerStack.close(first);
        expect(LayerStack.depth).toBe(0);
    });

    it("closes a layer that is not on top without disturbing the others", () => {
        const lower = vi.fn();
        const upper = vi.fn();
        const first = LayerStack.open(lower);
        const second = LayerStack.open(upper);

        LayerStack.close(first);
        expect(LayerStack.isTop(second)).toBe(true);

        LayerStack.close(second);
        LayerStack.close(second);
        expect(LayerStack.depth).toBe(0);
    });
});

describe("layers on screen", () => {
    it("closes the inner dialog on the first Escape and the outer one on the second", async () => {
        const outerclose = vi.fn();
        const innerclose = vi.fn();
        render(LayerHost, { outerclose, innerclose, withInner: true });

        await fireEvent.keyDown(window, { key: "Escape" });
        expect(innerclose).toHaveBeenCalledOnce();
        expect(outerclose).not.toHaveBeenCalled();

        await fireEvent.keyDown(window, { key: "Escape" });
        expect(outerclose).toHaveBeenCalledOnce();
        expect(innerclose).toHaveBeenCalledOnce();
    });

    it("closes only the menu when Escape is pressed in a menu inside a dialog", async () => {
        const outerclose = vi.fn();
        render(LayerHost, { outerclose });

        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        expect(screen.getByRole("menu")).toBeInTheDocument();

        await fireEvent.keyDown(window, { key: "Escape" });
        expect(screen.queryByRole("menu")).toBeNull();
        expect(outerclose).not.toHaveBeenCalled();

        await fireEvent.keyDown(window, { key: "Escape" });
        expect(outerclose).toHaveBeenCalledOnce();
    });

    it("keeps Tab inside the top dialog when focus is outside every card", async () => {
        render(LayerHost, { withInner: true });
        const outside = document.createElement("button");
        document.body.append(outside);
        outside.focus();

        await fireEvent.keyDown(outside, { key: "Tab" });

        expect(screen.getByRole("dialog", { name: "Inner" })).toContainElement(document.activeElement as HTMLElement);
        outside.remove();
        await tick();
    });
});
