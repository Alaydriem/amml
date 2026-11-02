import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import ChoiceList from "../../../src/components/setup/ChoiceList.svelte";
import { AdoptedInstanceRowRules } from "../../../src/components/setup/AdoptedInstanceRow";
import { firstRunAdoptEn } from "../../../src/text/en/firstRunAdopt";

const choices = [
    { id: "a", title: "First" },
    { id: "b", title: "Second" },
    { id: "c", title: "Third" },
];

function mount(value: string | undefined) {
    const onchange = vi.fn();
    render(ChoiceList, { heading: "Copy", choices, value, onchange });
    return onchange;
}

describe("ChoiceList", () => {
    it("asks for the pressed row and shows the value it was given", async () => {
        const onchange = mount("a");

        await fireEvent.click(screen.getByRole("radio", { name: /Second/ }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith("b");
        expect(screen.getByRole("radio", { name: /Second/ })).toHaveAttribute("aria-checked", "false");
        expect(screen.getByRole("radio", { name: /First/ })).toHaveAttribute("aria-checked", "true");
    });

    it("has one tab stop, on the selected row", () => {
        mount("b");
        const stops = screen.getAllByRole("radio").filter((row) => row.tabIndex === 0);
        expect(stops).toHaveLength(1);
        expect(stops[0]).toHaveAccessibleName(/Second/);
    });

    it("puts the tab stop on the first row when nothing is selected", () => {
        mount(undefined);
        expect(screen.getByRole("radio", { name: /First/ }).tabIndex).toBe(0);
    });

    it("moves the selection with the arrow keys and wraps at both ends", async () => {
        const onchange = mount("b");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /Second/ }), { key: "ArrowDown" });
        expect(onchange).toHaveBeenLastCalledWith("c");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /Second/ }), { key: "ArrowUp" });
        expect(onchange).toHaveBeenLastCalledWith("a");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /Third/ }), { key: "ArrowDown" });
        expect(onchange).toHaveBeenLastCalledWith("a");
    });

    it("goes to the first and last row with Home and End", async () => {
        const onchange = mount("b");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /Second/ }), { key: "End" });
        expect(onchange).toHaveBeenLastCalledWith("c");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /Second/ }), { key: "Home" });
        expect(onchange).toHaveBeenLastCalledWith("a");
    });

    it("moves focus to the row the arrow key chose", async () => {
        mount("a");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /First/ }), { key: "ArrowDown" });
        expect(screen.getByRole("radio", { name: /Second/ })).toHaveFocus();
    });

    it("ignores keys that do not move", async () => {
        const onchange = mount("a");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /First/ }), { key: "x" });
        expect(onchange).not.toHaveBeenCalled();
    });
});

describe("AdoptedInstanceRowRules.meta", () => {
    const text = firstRunAdoptEn.adopted;

    it("names an unknown AMML version", () => {
        expect(AdoptedInstanceRowRules.meta({ id: "a", name: "n", swatch: 1, modCount: 7 }, text)).toBe(
            "AMML unknown version · 7 mods",
        );
    });

    it("uses the singular for one mod", () => {
        expect(
            AdoptedInstanceRowRules.meta({ id: "a", name: "n", swatch: 1, ammlVersion: "26.60.5", modCount: 1 }, text),
        ).toBe("AMML 26.60.5 · 1 mod");
    });
});
