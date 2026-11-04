import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import Select from "../../../src/components/primitives/Select.svelte";

const options = [
    { id: "name", label: "Name" },
    { id: "updated", label: "Updated" },
];

describe("Select", () => {
    it("calls back with the picked id", async () => {
        const onchange = vi.fn();
        render(Select, { options, value: "name", label: "Sort", onchange });

        await fireEvent.change(screen.getByRole("combobox", { name: "Sort" }), { target: { value: "updated" } });

        expect(onchange).toHaveBeenCalledExactlyOnceWith("updated");
    });

    it("shows the prop again when the parent refuses the pick", async () => {
        render(Select, { options, value: "name", label: "Sort", onchange: vi.fn() });
        const select = screen.getByRole<HTMLSelectElement>("combobox", { name: "Sort" });

        await fireEvent.change(select, { target: { value: "updated" } });
        await tick();

        expect(select.value).toBe("name");
    });
});
