import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import Menu from "../../../src/components/primitives/Menu.svelte";
import { MenuRules } from "../../../src/components/primitives/Menu";

const items = [
    { id: "open", label: "Open folder" },
    { id: "log", label: "Show log" },
    { id: "delete", label: "Delete", tone: "fault" as const },
];

describe("Menu", () => {
    it("calls onselect with the item id and closes", async () => {
        const onselect = vi.fn();
        render(Menu, { items, label: "More", onselect });

        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        await fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));

        expect(onselect).toHaveBeenCalledExactlyOnceWith("delete");
        expect(screen.queryByRole("menu")).toBeNull();
    });

    it("closes on Escape without selecting", async () => {
        const onselect = vi.fn();
        render(Menu, { items, label: "More", onselect });

        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        expect(screen.getByRole("menu")).toBeInTheDocument();
        await fireEvent.keyDown(window, { key: "Escape" });

        expect(screen.queryByRole("menu")).toBeNull();
        expect(onselect).not.toHaveBeenCalled();
    });

    it("closes on a pointer press outside", async () => {
        render(Menu, { items, label: "More" });

        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        await fireEvent.pointerDown(document.body);

        expect(screen.queryByRole("menu")).toBeNull();
    });

    it("focuses the first item on open and moves with the arrows, Home and End", async () => {
        render(Menu, { items, label: "More" });
        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        await tick();
        expect(screen.getByRole("menuitem", { name: "Open folder" })).toHaveFocus();

        await fireEvent.keyDown(window, { key: "ArrowDown" });
        expect(screen.getByRole("menuitem", { name: "Show log" })).toHaveFocus();

        await fireEvent.keyDown(window, { key: "End" });
        expect(screen.getByRole("menuitem", { name: "Delete" })).toHaveFocus();

        await fireEvent.keyDown(window, { key: "Home" });
        expect(screen.getByRole("menuitem", { name: "Open folder" })).toHaveFocus();
    });

    it("moves to the last item with ArrowUp when no item has focus", async () => {
        render(Menu, { items, label: "More" });
        const trigger = screen.getByRole("button", { name: "More" });
        await fireEvent.click(trigger);
        await tick();
        trigger.focus();

        await fireEvent.keyDown(trigger, { key: "ArrowUp" });

        expect(screen.getByRole("menuitem", { name: "Delete" })).toHaveFocus();
    });

    it("closes when Tab leaves it and hands focus back to the button", async () => {
        render(Menu, { items, label: "More" });
        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        await tick();

        await fireEvent.keyDown(window, { key: "Tab" });

        expect(screen.queryByRole("menu")).toBeNull();
        expect(screen.getByRole("button", { name: "More" })).toHaveFocus();
    });

    it("closes when focus moves outside the menu", async () => {
        render(Menu, { items, label: "More" });
        const outside = document.createElement("button");
        document.body.append(outside);
        await fireEvent.click(screen.getByRole("button", { name: "More" }));
        await tick();

        outside.focus();
        await tick();

        expect(screen.queryByRole("menu")).toBeNull();
        outside.remove();
    });
});

describe("MenuRules", () => {
    it("opens on the first enabled item", () => {
        expect(MenuRules.first([{ id: "a", label: "A", disabled: true }, { id: "b", label: "B" }])).toBe(1);
        expect(MenuRules.first([])).toBe(-1);
    });

    it("skips disabled items for the arrow keys", () => {
        const list = [
            { id: "a", label: "A" },
            { id: "b", label: "B", disabled: true },
            { id: "c", label: "C" },
        ];
        expect(MenuRules.target(list, "ArrowDown", 0)).toBe(2);
        expect(MenuRules.target(list, "ArrowUp", 0)).toBe(2);
        expect(MenuRules.target(list, "ArrowRight", 0)).toBeNull();
    });
});
