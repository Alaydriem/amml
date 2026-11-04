import { afterEach, describe, expect, it } from "vitest";
import { I18n, type LanguagePack } from "#lib/i18n/index.ts";

const german: LanguagePack = {
    v: 1,
    locale: "de",
    plural: ["one", "other"],
    m: {
        Hello: "Hallo",
        "{n} mod": ["{n} Mod", "{n} Mods"],
        "Open {name}": "{name} öffnen",
    },
};

describe("I18n", () => {
    afterEach(() => I18n.adopt(null));

    it("returns the English message id when no pack is loaded", () => {
        expect(I18n.t("Hello")).toBe("Hello");
        expect(I18n.tn("{n} mod", "{n} mods", 1)).toBe("{n} mod");
        expect(I18n.tn("{n} mod", "{n} mods", 2)).toBe("{n} mods");
        expect(I18n.locale).toBe("en");
    });

    it("fills slots with tf and leaves unknown slots visible", () => {
        expect(I18n.tf("Open {name}", { name: "Peek" })).toBe("Open Peek");
        expect(I18n.tf("Open {name} in {where}", { name: "Peek" })).toBe("Open Peek in {where}");
        I18n.adopt(german);
        expect(I18n.tf("Open {name}", { name: "Peek" })).toBe("Peek öffnen");
    });

    it("picks the plural form from the pack's categories", () => {
        I18n.adopt(german);
        expect(I18n.locale).toBe("de");
        expect(I18n.tn("{n} mod", "{n} mods", 1)).toBe("{n} Mod");
        expect(I18n.tn("{n} mod", "{n} mods", 5)).toBe("{n} Mods");
    });

    it("falls back to English for a message the pack lacks", () => {
        I18n.adopt(german);
        expect(I18n.t("Unknown")).toBe("Unknown");
    });
});
