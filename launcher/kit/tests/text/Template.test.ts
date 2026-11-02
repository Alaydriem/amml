import { describe, expect, it } from "vitest";
import { Template } from "../../src/text/Template";

describe("Template", () => {
    it("fills every slot and leaves an unknown slot visible", () => {
        expect(Template.fill("{count} work with api {api} · {missing}", { count: "1,412", api: 2 })).toBe(
            "1,412 work with api 2 · {missing}",
        );
    });

    it("picks the plural form by count", () => {
        const forms = { one: "{count} update", other: "{count} updates" };
        expect(Template.plural(forms, 1)).toBe("1 update");
        expect(Template.plural(forms, 3)).toBe("3 updates");
        expect(Template.plural(forms, 0)).toBe("0 updates");
    });
});

describe("Template.parts", () => {
    it("splits a template at its strong marks and fills the slots of each run", () => {
        expect(Template.parts("**{count} updates available** in {where}", { count: 3, where: "2 instances" })).toEqual([
            { text: "3 updates available", strong: true },
            { text: " in 2 instances", strong: false },
        ]);
    });

    it("keeps several strong runs apart and drops empty runs", () => {
        expect(Template.parts("**AMML {amml}** and **{mods}**", { amml: "26.60.5", mods: "6 mods" })).toEqual([
            { text: "AMML 26.60.5", strong: true },
            { text: " and ", strong: false },
            { text: "6 mods", strong: true },
        ]);
    });

    it("returns one plain run for a template with no marks", () => {
        expect(Template.parts("Nothing to {do}", { do: "see" })).toEqual([{ text: "Nothing to see", strong: false }]);
    });

    it("leaves a mark with no partner in the text", () => {
        expect(Template.parts("5 ** 2")).toEqual([{ text: "5 ** 2", strong: false }]);
    });

    it("reads marks from the template only, never from a value", () => {
        expect(Template.parts("name: {name}", { name: "**bold**" })).toEqual([{ text: "name: **bold**", strong: false }]);
    });
});
