import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import VersionList from "../../../src/components/instances/VersionList.svelte";
import { VersionRowRules } from "../../../src/components/instances/VersionRow";
import { InstanceFixtures } from "../../../src/fixtures/instances";
import { newInstanceEn } from "../../../src/text/en/newInstance";

const text = newInstanceEn.versionRow;
const versions = InstanceFixtures.versions.filter((version) => version.channel === "release");

function mount(value: string | undefined) {
    const onchange = vi.fn();
    render(VersionList, { versions, text, label: "Versions", value, onchange });
    return onchange;
}

describe("VersionList", () => {
    it("selects a supported version and calls back with it", async () => {
        const onchange = mount("1.26.60.5");
        await fireEvent.click(screen.getByRole("radio", { name: /1\.26\.51\.1/ }));
        expect(onchange).toHaveBeenCalledExactlyOnceWith("1.26.51.1");
    });

    it("does not select an unsupported version", async () => {
        const onchange = mount("1.26.60.5");
        const row = screen.getByRole("radio", { name: /1\.26\.50\.2/ });
        await fireEvent.click(row);
        expect(row).toBeDisabled();
        expect(onchange).not.toHaveBeenCalled();
    });

    it("shows the value it was given and does not change it itself", async () => {
        mount("1.26.60.5");
        await fireEvent.click(screen.getByRole("radio", { name: /1\.26\.51\.1/ }));
        expect(screen.getByRole("radio", { name: /1\.26\.60\.5/ })).toHaveAttribute("aria-checked", "true");
        expect(screen.getByRole("radio", { name: /1\.26\.51\.1/ })).toHaveAttribute("aria-checked", "false");
    });

    it("has one tab stop, on the selected row", () => {
        mount("1.26.52.3");
        const stops = screen.getAllByRole("radio").filter((row) => row.tabIndex === 0);
        expect(stops).toHaveLength(1);
        expect(stops[0]).toHaveAccessibleName(/1\.26\.52\.3/);
    });

    it("moves the selection down with ArrowDown and skips an unsupported row", async () => {
        const onchange = mount("1.26.52.3");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.52\.3/ }), { key: "ArrowDown" });
        expect(onchange).toHaveBeenLastCalledWith("1.26.51.1");

        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.51\.1/ }), { key: "ArrowDown" });
        expect(onchange).toHaveBeenLastCalledWith("1.26.60.5");
    });

    it("wraps ArrowUp from the first row to the last supported one", async () => {
        const onchange = mount("1.26.60.5");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.60\.5/ }), { key: "ArrowUp" });
        expect(onchange).toHaveBeenCalledExactlyOnceWith("1.26.51.1");
    });

    it("moves focus with the selection", async () => {
        mount("1.26.52.3");
        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.52\.3/ }), { key: "ArrowDown" });
        expect(screen.getByRole("radio", { name: /1\.26\.51\.1/ })).toHaveFocus();
    });
});

describe("VersionRowRules", () => {
    it("picks the pill from the state", () => {
        expect(VersionRowRules.pill(versions[0]!, text)).toEqual({ tone: "ok", label: "Installed" });
        expect(VersionRowRules.pill(versions[2]!, text)).toEqual({ tone: "brand", label: "~4.1 GB download" });
        expect(VersionRowRules.pill(versions[3]!, text)).toEqual({ tone: "muted", label: "Not supported" });
    });
});
