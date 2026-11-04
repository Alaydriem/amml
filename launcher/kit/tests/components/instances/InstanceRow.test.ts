import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import InstanceRow from "../../../src/components/instances/InstanceRow.svelte";
import { InstanceRowRules } from "../../../src/components/instances/InstanceRow";
import type { Instance } from "../../../src/model";
import { instanceMenuEn, instanceRowEn } from "../../../src/text/en/instances";

const idle: Instance = {
    id: "a",
    name: "1.26.60.05",
    swatch: 3,
    gameVersion: "1.26.60.5",
    ammlVersion: "26.60.5",
    modCount: 7,
    dataMode: "shared",
    folder: "instances\\a",
    activity: "last played 2 h ago",
    running: false,
    updateCount: 2,
};

function mount(instance: Instance, extra: Record<string, unknown> = {}) {
    const onlaunch = vi.fn();
    const onopen = vi.fn();
    const onmenu = vi.fn();
    render(InstanceRow, {
        instance,
        text: instanceRowEn,
        menuItems: instanceMenuEn,
        onlaunch,
        onopen,
        onmenu,
        ...extra,
    });
    return { onlaunch, onopen, onmenu };
}

describe("InstanceRow", () => {
    it("calls onlaunch with the instance id when Launch is pressed", async () => {
        const { onlaunch } = mount(idle, { emphasis: true });
        await fireEvent.click(screen.getByRole("button", { name: "Launch" }));
        expect(onlaunch).toHaveBeenCalledExactlyOnceWith("a");
    });

    it("calls onlaunch from the outlined Launch of a row that is not emphasised", async () => {
        const { onlaunch } = mount(idle, { emphasis: false });
        await fireEvent.click(screen.getByRole("button", { name: "Launch" }));
        expect(onlaunch).toHaveBeenCalledExactlyOnceWith("a");
    });

    it("shows a running instance as a status, with no button for Launch or Running", () => {
        mount({ ...idle, running: true });
        expect(screen.queryByRole("button", { name: "Launch" })).toBeNull();
        expect(screen.queryByRole("button", { name: "Running" })).toBeNull();
        expect(screen.getByRole("status")).toHaveTextContent("Running");
    });

    it("opens the instance from its name", async () => {
        const { onopen } = mount(idle);
        await fireEvent.click(screen.getByRole("button", { name: /1\.26\.60\.05/ }));
        expect(onopen).toHaveBeenCalledExactlyOnceWith("a");
    });

    it("reports the row menu choice with the instance id and the item id", async () => {
        const { onmenu } = mount(idle);
        await fireEvent.click(screen.getByRole("button", { name: "More actions" }));
        await fireEvent.click(screen.getByRole("menuitem", { name: "Delete" }));
        expect(onmenu).toHaveBeenCalledExactlyOnceWith("a", "delete");
    });

    it("shows the update count, or Up to date at zero", () => {
        mount(idle);
        expect(screen.getByText("2 updates")).toBeInTheDocument();
        expect(InstanceRowRules.updateLabel({ ...idle, updateCount: 1 }, instanceRowEn)).toBe("1 update");
        expect(InstanceRowRules.updateLabel({ ...idle, updateCount: 0 }, instanceRowEn)).toBeNull();
    });

    it("builds the activity line from the data mode and the activity", () => {
        expect(InstanceRowRules.activityLine(idle, instanceRowEn)).toBe("Shared data · last played 2 h ago");
        expect(InstanceRowRules.activityLine({ ...idle, dataMode: "own" }, instanceRowEn)).toBe(
            "Own data folder · last played 2 h ago",
        );
    });
});
