import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

// The TypeScript `LanguagePack` is written by hand, so this follows the Rust type it mirrors.
describe("LanguagePack", () => {
    it("has the fields of the amml-i18n pack", () => {
        const rust = readFileSync(join(process.cwd(), "..", "..", "i18n", "runtime", "src", "pack", "language_pack.rs"), "utf8");
        const ts = readFileSync(join(process.cwd(), "src", "lib", "i18n", "LanguagePack.ts"), "utf8");

        const rustFields = [...rust.matchAll(/pub (\w+): /g)].map((match) => match[1] ?? "");
        for (const field of rustFields) expect(ts, field).toMatch(new RegExp(field + "\\??:"));
        expect(rustFields.sort()).toEqual(["locale", "m", "plural", "v"]);
    });
});
