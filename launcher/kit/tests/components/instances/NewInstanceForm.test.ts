import { fireEvent, render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { createRawSnippet } from "svelte";
import { describe, expect, it, vi } from "vitest";
import NewInstanceForm from "../../../src/components/instances/NewInstanceForm.svelte";
import { NewInstanceFormRules } from "../../../src/components/instances/NewInstanceForm";

const text = {
    nameLabel: "Instance name",
    folderHint: "Folder: {folder}",
    worldsLabel: "Worlds and packs",
    shared: "Shared",
    own: "Own folder",
    cancel: "Cancel",
    create: "Create",
};

const summary = createRawSnippet(() => ({ render: () => "<span>Summary</span>" }));

function mount(name: string, extra: Record<string, unknown> = {}) {
    const oncreate = vi.fn();
    const ondatamodechange = vi.fn();
    render(NewInstanceForm, {
        name,
        dataMode: "shared",
        takenNames: ["1.26.60.05", "scenes"],
        folder: "instances\\x",
        text,
        summary,
        oncreate,
        ondatamodechange,
        ...extra,
    });
    return { oncreate, ondatamodechange, create: screen.getByRole("button", { name: "Create" }) };
}

describe("NewInstanceForm", () => {
    it("disables Create for an empty name", () => {
        expect(mount("").create).toBeDisabled();
    });

    it("disables Create for a name of spaces", () => {
        expect(mount("   ").create).toBeDisabled();
    });

    it("disables Create for a taken name, ignoring case and outer spaces", () => {
        expect(mount(" SCENES ").create).toBeDisabled();
    });

    it("disables Create for a name Windows cannot use as a folder", () => {
        expect(mount("con").create).toBeDisabled();
    });

    it("enables Create for a free name", () => {
        expect(mount("players").create).toBeEnabled();
    });

    it("holds Create back when the caller says it cannot submit", () => {
        expect(mount("players", { canSubmit: false }).create).toBeDisabled();
    });

    it("hands back the trimmed name and the worlds mode it was given", async () => {
        const { oncreate, create } = mount("  players  ", { dataMode: "own" });
        await fireEvent.click(create);
        expect(oncreate).toHaveBeenCalledExactlyOnceWith({ name: "players", dataMode: "own" });
    });

    it("asks for the other worlds mode and leaves the shown mode to the parent", async () => {
        const { ondatamodechange } = mount("players");
        await fireEvent.click(screen.getByRole("radio", { name: "Own folder" }));
        expect(ondatamodechange).toHaveBeenCalledExactlyOnceWith("own");
        expect(screen.getByRole("radio", { name: "Shared" })).toHaveAttribute("aria-checked", "true");
    });

    it("creates once when Enter is pressed in the name field", async () => {
        const { oncreate } = mount("players");
        await userEvent.type(screen.getByRole("textbox", { name: "Instance name" }), "{Enter}");
        expect(oncreate).toHaveBeenCalledExactlyOnceWith({ name: "players", dataMode: "shared" });
    });

    it("creates once when the form is submitted", async () => {
        const { oncreate, create } = mount("players");
        await fireEvent.submit(create.closest("form")!);
        expect(oncreate).toHaveBeenCalledOnce();
    });

    it("does not call back when Create is disabled", async () => {
        const { oncreate, create } = mount("scenes");
        await fireEvent.click(create);
        expect(oncreate).not.toHaveBeenCalled();
    });

    it("does not create from Enter when the name is not allowed", async () => {
        const { oncreate } = mount("scenes");
        await userEvent.type(screen.getByRole("textbox", { name: "Instance name" }), "{Enter}");
        expect(oncreate).not.toHaveBeenCalled();
    });
});

describe("NewInstanceFormRules.canCreate", () => {
    it("accepts a name that only extends a taken one", () => {
        expect(NewInstanceFormRules.canCreate("scenes-2", ["scenes"])).toBe(true);
    });

    it("treats a name with trailing dots or spaces as the name without them", () => {
        expect(NewInstanceFormRules.canCreate("scenes.", ["scenes"])).toBe(false);
        expect(NewInstanceFormRules.canCreate("scenes . ", ["scenes"])).toBe(false);
        expect(NewInstanceFormRules.canCreate("scenes", ["scenes."])).toBe(false);
    });

    it("refuses a name that is only dots and spaces", () => {
        expect(NewInstanceFormRules.canCreate("...", [])).toBe(false);
        expect(NewInstanceFormRules.canCreate(" . ", [])).toBe(false);
    });

    it.each(["a<b", "a>b", "a:b", 'a"b', "a/b", "a\\b", "a|b", "a?b", "a*b", "a\u0007b", "a\tb"])(
        "refuses the character in %j",
        (name) => {
            expect(NewInstanceFormRules.canCreate(name, [])).toBe(false);
        },
    );

    it.each(["CON", "con", "Prn", "AUX", "nul", "COM1", "com9", "LPT1", "lpt9", "con.txt", "NUL.1.2", "con ."])(
        "refuses the reserved name %j",
        (name) => {
            expect(NewInstanceFormRules.canCreate(name, [])).toBe(false);
        },
    );

    it.each(["COM0", "COM10", "LPT0", "console", "con-1", "a.con", "1.26.60.05"])("accepts %j", (name) => {
        expect(NewInstanceFormRules.canCreate(name, [])).toBe(true);
    });
});

describe("NewInstanceForm problem", () => {
    it("shows the problem under the name field and holds Create back while it is set", async () => {
        const { oncreate } = mount("con", { problem: "Use another name." });

        expect(screen.getByRole("alert")).toHaveTextContent("Use another name.");
        const create = screen.getByRole("button", { name: "Create" });
        expect(create).toBeDisabled();
        await fireEvent.click(create);
        expect(oncreate).not.toHaveBeenCalled();
    });

    it("draws no problem line and allows Create for a good name", () => {
        mount("players");

        expect(screen.queryByRole("alert")).toBeNull();
        expect(screen.getByRole("button", { name: "Create" })).toBeEnabled();
    });
});
