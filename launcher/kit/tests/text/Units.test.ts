import { describe, expect, it } from "vitest";
import { unitsEn } from "../../src/text/en/units";
import { Units } from "../../src/text/Units";

const units = new Units(unitsEn);

describe("Units.size", () => {
    it("shows GB with two decimals from 1 GB up", () => {
        expect(units.size(2_060_000_000)).toBe("2.06 GB");
        expect(units.size(1_000_000_000)).toBe("1.00 GB");
    });

    it("never shows 1000 MB for a size just under 1 GB", () => {
        expect(units.size(999_999_999)).toBe("1.00 GB");
        expect(units.size(999_960_000)).toBe("1.00 GB");
        expect(units.size(999_000_000)).toBe("999.0 MB");
    });

    it("shows KB below 0.1 MB and MB from there", () => {
        expect(units.size(45_000)).toBe("45 KB");
        expect(units.size(99_999)).toBe("100 KB");
        expect(units.size(100_000)).toBe("0.1 MB");
        expect(units.size(0)).toBe("0 KB");
    });

    it("forces a unit and sets the digits", () => {
        expect(units.size(20_000_000, { unit: "gb" })).toBe("0.02 GB");
        expect(units.size(4_100_000_000, { digits: 1 })).toBe("4.1 GB");
        expect(units.size(612_000_000, { digits: 1 })).toBe("612.0 MB");
    });

    it("drops trailing zeros when asked", () => {
        expect(units.size(1_900_000_000, { trim: true })).toBe("1.9 GB");
        expect(units.size(3_000_000_000, { trim: true })).toBe("3 GB");
    });

    it("gives the number and the unit apart", () => {
        expect(units.sizeParts(3_340_000_000, { trim: true })).toEqual({ value: "3.34", unit: "GB" });
    });

    it("uses the words of the text it was built with", () => {
        const german = new Units({ ...unitsEn, gigabytes: "{n} Gigabyte" });
        expect(german.size(2_000_000_000)).toBe("2.00 Gigabyte");
    });
});

describe("Units.sizeOf", () => {
    it("formats progress in the unit of the total", () => {
        expect(units.sizeOf(1_280_000_000, 2_060_000_000)).toBe("1.28 / 2.06 GB");
        expect(units.sizeOf(300_000, 600_000)).toBe("0.3 / 0.6 MB");
        expect(units.sizeOf(10_000, 80_000)).toBe("10 / 80 KB");
    });
});

describe("Units.rate", () => {
    it("rounds to whole MB/s from 1 MB/s", () => {
        expect(units.rate(38_400_000)).toBe("38 MB/s");
        expect(units.rate(1_000_000)).toBe("1 MB/s");
    });

    it("keeps one decimal below 1 MB/s", () => {
        expect(units.rate(400_000)).toBe("0.4 MB/s");
        expect(units.rate(0)).toBe("0.0 MB/s");
    });
});

describe("Units.duration", () => {
    it("formats durations across the minute and hour edges", () => {
        expect(units.duration(20)).toBe("20 s");
        expect(units.duration(59.6)).toBe("1 min");
        expect(units.duration(3600)).toBe("1 h");
        expect(units.duration(3900)).toBe("1 h 5 min");
        expect(units.duration(-4)).toBe("0 s");
    });
});

describe("Units.percent and Units.count", () => {
    it("clamps the percent and survives a zero total", () => {
        expect(Units.percent(62, 100)).toBe(62);
        expect(Units.percent(120, 100)).toBe(100);
        expect(Units.percent(1, 0)).toBe(0);
    });

    it("groups digits by the locale of the text", () => {
        expect(units.count(1412)).toBe("1,412");
        expect(new Units({ ...unitsEn, locale: "de-DE" }).count(1412)).toBe("1.412");
    });
});
