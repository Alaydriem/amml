import { describe, expect, it } from "vitest";
import { RovingFocus } from "../../../src/components/primitives/RovingFocus";

describe("RovingFocus", () => {
    it("skips disabled items when moving", () => {
        const disabled = (index: number) => index === 1;
        expect(RovingFocus.next(3, 0, 1, disabled)).toBe(2);
        expect(RovingFocus.next(3, 0, -1, disabled)).toBe(2);
    });

    it("lands on the last item for ArrowUp when nothing has focus", () => {
        expect(RovingFocus.next(4, -1, -1)).toBe(3);
        expect(RovingFocus.target("ArrowUp", -1, 4, { axis: "vertical" })).toBe(3);
    });

    it("lands on the first item for ArrowDown when nothing has focus", () => {
        expect(RovingFocus.target("ArrowDown", -1, 4, { axis: "vertical" })).toBe(0);
    });

    it("wraps at both ends", () => {
        expect(RovingFocus.next(3, 2, 1)).toBe(0);
        expect(RovingFocus.next(3, 0, -1)).toBe(2);
    });

    it("sends Home and End to the first and last enabled item", () => {
        const disabled = (index: number) => index === 0 || index === 3;
        expect(RovingFocus.target("Home", 2, 4, { disabled })).toBe(1);
        expect(RovingFocus.target("End", 1, 4, { disabled })).toBe(2);
    });

    it("answers only the arrows of its axis", () => {
        expect(RovingFocus.target("ArrowLeft", 1, 3, { axis: "vertical" })).toBeNull();
        expect(RovingFocus.target("ArrowDown", 1, 3, { axis: "horizontal" })).toBeNull();
        expect(RovingFocus.target("ArrowDown", 1, 3, { axis: "both" })).toBe(2);
    });

    it("moves nowhere when every item is disabled or the list is empty", () => {
        expect(RovingFocus.next(2, 0, 1, () => true)).toBe(-1);
        expect(RovingFocus.target("ArrowDown", 0, 0)).toBeNull();
    });

    it("leaves other keys alone", () => {
        expect(RovingFocus.target("a", 0, 3)).toBeNull();
    });

    it("makes the first item the tab stop when nothing is selected", () => {
        expect(RovingFocus.stop(-1)).toBe(0);
        expect(RovingFocus.stop(2)).toBe(2);
    });
});
