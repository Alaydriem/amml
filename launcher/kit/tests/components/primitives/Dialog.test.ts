import { fireEvent, render, screen } from "@testing-library/svelte";
import { createRawSnippet } from "svelte";
import { describe, expect, it, vi } from "vitest";
import Dialog from "../../../src/components/primitives/Dialog.svelte";
import { DialogFocus } from "../../../src/components/primitives/Dialog";

const body = createRawSnippet(() => ({ render: () => "<p>Body text</p>" }));
const actions = createRawSnippet(() => ({
    render: () => '<span><button type="button">Keep it</button><button type="button">Delete</button></span>',
}));

describe("Dialog", () => {
    it("calls onclose on Escape", async () => {
        const onclose = vi.fn();
        render(Dialog, { open: true, title: "Delete?", onclose, children: body, actions });

        await fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });

        expect(onclose).toHaveBeenCalledOnce();
    });

    it("moves focus to the first control when it opens", () => {
        render(Dialog, { open: true, title: "Delete?", children: body, actions });

        expect(screen.getByRole("button", { name: "Keep it" })).toHaveFocus();
    });

    it("draws nothing while closed", () => {
        render(Dialog, { open: false, title: "Delete?", children: body });

        expect(screen.queryByRole("dialog")).toBeNull();
    });

    it("is named by its title", () => {
        render(Dialog, { open: true, title: "Delete?", children: body });

        expect(screen.getByRole("dialog", { name: "Delete?" })).toHaveAttribute("aria-modal", "true");
    });
});

describe("DialogFocus.wrap", () => {
    function card(): { card: HTMLElement; first: HTMLElement; last: HTMLElement; outside: HTMLElement } {
        const root = document.createElement("div");
        root.innerHTML = '<button id="a">A</button><button id="b">B</button><button id="c">C</button>';
        const outside = document.createElement("button");
        document.body.append(root, outside);
        return {
            card: root,
            first: root.querySelector("#a") as HTMLElement,
            last: root.querySelector("#c") as HTMLElement,
            outside,
        };
    }

    it("sends focus from outside the card to the first control", () => {
        const { card: root, first, outside } = card();
        expect(DialogFocus.wrap(root, outside, false)).toBe(first);
    });

    it("sends Shift+Tab from outside the card to the last control", () => {
        const { card: root, last, outside } = card();
        expect(DialogFocus.wrap(root, outside, true)).toBe(last);
    });

    it("wraps at the ends and lets the browser move in between", () => {
        const { card: root, first, last } = card();
        expect(DialogFocus.wrap(root, last, false)).toBe(first);
        expect(DialogFocus.wrap(root, first, true)).toBe(last);
        expect(DialogFocus.wrap(root, root.querySelector("#b"), false)).toBeNull();
    });
});
