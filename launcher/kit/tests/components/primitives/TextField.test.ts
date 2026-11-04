import { fireEvent, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { describe, expect, it, vi } from "vitest";
import TextField from "../../../src/components/primitives/TextField.svelte";

describe("TextField", () => {
    it("calls back with the typed text", async () => {
        const oninput = vi.fn();
        render(TextField, { value: "", label: "Name", oninput });

        await fireEvent.input(screen.getByRole("textbox", { name: "Name" }), { target: { value: "abc" } });

        expect(oninput).toHaveBeenCalledExactlyOnceWith("abc");
    });

    it("shows the prop again when the parent refuses the text", async () => {
        render(TextField, { value: "ab", label: "Name", oninput: vi.fn() });
        const input = screen.getByRole<HTMLInputElement>("textbox", { name: "Name" });

        await fireEvent.input(input, { target: { value: "abc" } });
        await tick();

        expect(input.value).toBe("ab");
    });

    it("calls onsubmit with the text on Enter", async () => {
        const onsubmit = vi.fn();
        render(TextField, { value: "abc", label: "Name", onsubmit });

        await fireEvent.keyDown(screen.getByRole("textbox", { name: "Name" }), { key: "Enter" });

        expect(onsubmit).toHaveBeenCalledExactlyOnceWith("abc");
    });
});
