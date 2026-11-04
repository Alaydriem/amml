import { describe, expect, it } from "vitest";
import { DialogFixtures } from "../../src/fixtures/dialogs";
import { faultsEn } from "../../src/text/en/faults";

describe("faultsEn", () => {
    it("has a text and a fixture for every fault kind of the banner list", () => {
        for (const kind of DialogFixtures.faultOrder) {
            expect(faultsEn[kind].title, kind).not.toBe("");
            expect(DialogFixtures.faults[kind].kind).toBe(kind);
        }
    });

    it("says honestly that the Store version was not put back, and keeps that out of the general fault", () => {
        expect(faultsEn["restore-failed"].title).toBe("Your Store version was not put back.");
        expect(faultsEn["restore-failed"].body).toBe(
            "Put it back with the command in the details, or reinstall Minecraft from the Store.",
        );
        expect(faultsEn["restore-failed"].action).toBe("Copy details");
        expect(faultsEn["operation-failed"].body).toBe("The details are below.");
        expect(DialogFixtures.faults["restore-failed"].code).toContain("msixvc");
    });
});
