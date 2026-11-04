import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import FirstRunAdoptScreen from "../../src/screens/FirstRunAdoptScreen.svelte";
import InstallProgressScreen from "../../src/screens/InstallProgressScreen.svelte";
import InstancesScreen from "../../src/screens/InstancesScreen.svelte";
import { InstancesScreenRules } from "../../src/screens/InstancesScreen";
import FirstRunChecksScreen from "../../src/screens/FirstRunChecksScreen.svelte";
import NewInstanceScreen from "../../src/screens/NewInstanceScreen.svelte";
import { NewInstanceScreenRules } from "../../src/screens/NewInstanceScreen";
import { InstallFixtures } from "../../src/fixtures/install";
import { InstanceFixtures } from "../../src/fixtures/instances";
import { SetupFixtures } from "../../src/fixtures/setup";
import { firstRunChecksEn } from "../../src/text/en/firstRunChecks";
import { faultsEn } from "../../src/text/en/faults";
import { firstRunAdoptEn } from "../../src/text/en/firstRunAdopt";
import { installProgressEn } from "../../src/text/en/installProgress";
import { instanceMenuEn, instancesEn } from "../../src/text/en/instances";
import { newInstanceEn } from "../../src/text/en/newInstance";

describe("InstancesScreen", () => {
    it("totals the updates and the instances that have them", () => {
        expect(InstancesScreenRules.updateTotals(InstanceFixtures.instances)).toEqual({ updates: 3, instances: 2 });
    });

    it("shows the banner with the totals and hides it when nothing has an update", () => {
        const { unmount } = render(InstancesScreen, {
            instances: InstanceFixtures.instances,
            text: instancesEn,
            menuItems: instanceMenuEn,
        });
        expect(screen.getByText("3 updates available")).toBeInTheDocument();
        expect(screen.getByText(/in 2 instances\./)).toBeInTheDocument();
        unmount();

        render(InstancesScreen, {
            instances: InstanceFixtures.instances.map((instance) => ({ ...instance, updateCount: 0 })),
            text: instancesEn,
            menuItems: instanceMenuEn,
        });
        expect(screen.queryByText(/updates? available/)).toBeNull();
    });

    it("shows the empty state with no instance, and its New instance button calls onnew", async () => {
        const onnew = vi.fn();
        render(InstancesScreen, { instances: [], text: instancesEn, menuItems: instanceMenuEn, onnew });
        expect(screen.getByRole("heading", { name: "No instances yet." })).toBeInTheDocument();
        expect(screen.getByText("Instances · 0")).toBeInTheDocument();
        expect(screen.queryByRole("list", { name: "Instances" })).toBeNull();
        expect(screen.queryByText(/updates? available/)).toBeNull();

        await fireEvent.click(screen.getByRole("button", { name: "New instance" }));

        expect(onnew).toHaveBeenCalledOnce();
    });

    it("lists the instances and no empty state when there are some", () => {
        render(InstancesScreen, { instances: InstanceFixtures.instances, text: instancesEn, menuItems: instanceMenuEn });
        expect(screen.getByRole("list", { name: "Instances" })).toBeInTheDocument();
        expect(screen.queryByText("No instances yet.")).toBeNull();
    });
});

describe("NewInstanceScreenRules", () => {
    const versions = InstanceFixtures.versions;

    it("lists only the versions of the channel", () => {
        expect(NewInstanceScreenRules.visible(versions, "preview").map((version) => version.version)).toEqual([
            "1.26.70.20",
            "1.26.70.10",
        ]);
    });

    it("allows only a listed, supported version of the channel", () => {
        expect(NewInstanceScreenRules.selectable(versions, "release", "1.26.60.5")).toBe(true);
        expect(NewInstanceScreenRules.selectable(versions, "release", "1.26.50.2")).toBe(false);
        expect(NewInstanceScreenRules.selectable(versions, "release", "1.26.70.20")).toBe(false);
        expect(NewInstanceScreenRules.selectable(versions, "release", undefined)).toBe(false);
    });
});

describe("NewInstanceScreen", () => {
    const base = {
        versions: InstanceFixtures.versions,
        takenNames: [],
        summary: InstanceFixtures.summary,
        text: newInstanceEn,
        channel: "release" as const,
        selectedVersion: "1.26.60.5",
        name: " players ",
        dataMode: "shared" as const,
    };

    it("draws the summary and sends the chosen version with the trimmed name", async () => {
        const oncreate = vi.fn();
        render(NewInstanceScreen, { ...base, oncreate });
        expect(screen.getByText("AMML 26.60.5")).toBeInTheDocument();
        expect(screen.getByText("6 default mods")).toBeInTheDocument();
        expect(screen.getByText("About 1.9 GB on disk.")).toBeInTheDocument();

        await fireEvent.click(screen.getByRole("button", { name: "Create" }));

        expect(oncreate).toHaveBeenCalledExactlyOnceWith({ name: "players", dataMode: "shared", version: "1.26.60.5" });
    });

    it("asks for a channel change and keeps the channel it was given", async () => {
        const onchannelchange = vi.fn();
        render(NewInstanceScreen, { ...base, onchannelchange });
        await fireEvent.click(screen.getByRole("radio", { name: "Preview" }));
        expect(onchannelchange).toHaveBeenCalledExactlyOnceWith("preview");
        expect(screen.getByRole("radio", { name: "Release" })).toHaveAttribute("aria-checked", "true");
    });

    it("shows the problem and keeps Create off", () => {
        render(NewInstanceScreen, { ...base, problem: "Use another name." });
        expect(screen.getByRole("alert")).toHaveTextContent("Use another name.");
        expect(screen.getByRole("button", { name: "Create" })).toBeDisabled();
    });

    it("hides the release and preview switch when preview builds are off", () => {
        render(NewInstanceScreen, { ...base, showPreview: false });
        expect(screen.queryByRole("radio", { name: "Preview" })).toBeNull();
        expect(screen.queryByRole("radio", { name: "Release" })).toBeNull();
    });

    it("asks for a version change when the list is moved with the arrow keys", async () => {
        const onversionchange = vi.fn();
        render(NewInstanceScreen, { ...base, selectedVersion: "1.26.52.3", onversionchange });
        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.52\.3/ }), { key: "ArrowDown" });
        expect(onversionchange).toHaveBeenCalledExactlyOnceWith("1.26.51.1");
    });

    it("does not create without a chosen version", async () => {
        const oncreate = vi.fn();
        render(NewInstanceScreen, { ...base, selectedVersion: undefined, oncreate });
        expect(screen.getByRole("button", { name: "Create" })).toBeDisabled();
    });
});

describe("FirstRunAdoptScreen", () => {
    it("asks for another source with the arrow keys and keeps the one it was given", async () => {
        const onselect = vi.fn();
        render(FirstRunAdoptScreen, {
            instances: SetupFixtures.adopted,
            choices: SetupFixtures.choices,
            selected: "instance",
            text: firstRunAdoptEn,
            onselect,
        });
        await fireEvent.keyDown(screen.getByRole("radio", { name: /1\.26\.60\.05 instance/ }), { key: "ArrowDown" });
        expect(onselect).toHaveBeenCalledExactlyOnceWith("store");
        expect(screen.getByRole("radio", { name: /1\.26\.60\.05 instance/ })).toHaveAttribute("aria-checked", "true");
    });
});

describe("InstallProgressScreen", () => {
    const running = {
        name: InstallFixtures.name,
        steps: InstallFixtures.steps,
        progress: InstallFixtures.progress,
        stepNumber: InstallFixtures.stepNumber,
        stepTotal: InstallFixtures.stepTotal,
        text: installProgressEn,
    };

    const failed = {
        ...running,
        steps: InstallFixtures.failedSteps,
        fault: InstallFixtures.fault,
        faultText: faultsEn[InstallFixtures.fault.kind],
        restored: true,
    };

    it("shows the progress and Cancel install while the install runs", async () => {
        const oncancel = vi.fn();
        render(InstallProgressScreen, { ...running, oncancel });
        expect(screen.getByRole("heading", { name: "Creating 1.26.51.1-players" })).toBeInTheDocument();
        expect(screen.getByText("Step 3 of 6")).toBeInTheDocument();
        expect(screen.getByRole("progressbar")).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Try again" })).toBeNull();

        await fireEvent.click(screen.getByRole("button", { name: "Cancel install" }));

        expect(oncancel).toHaveBeenCalledOnce();
    });

    it("shows the fault, the restored note and the retry action when it failed", async () => {
        const onretry = vi.fn();
        const onshowlog = vi.fn();
        render(InstallProgressScreen, { ...failed, onretry, onshowlog });
        expect(screen.getByRole("heading", { name: "Could not create 1.26.51.1-players" })).toBeInTheDocument();
        expect(screen.getByText("Stopped at step 3 of 6")).toBeInTheDocument();
        expect(screen.getByText("Download failed on both mirrors.")).toBeInTheDocument();
        expect(screen.getByText("Your Store version was put back.")).toBeInTheDocument();
        expect(screen.queryByRole("progressbar")).toBeNull();
        expect(screen.queryByRole("button", { name: "Cancel install" })).toBeNull();

        await fireEvent.click(screen.getByRole("button", { name: "Try again" }));
        await fireEvent.click(screen.getByRole("button", { name: "Show log" }));

        expect(onretry).toHaveBeenCalledOnce();
        expect(onshowlog).toHaveBeenCalledOnce();
    });

    it("leaves out the restored note unless the app says the Store version is back", () => {
        render(InstallProgressScreen, { ...failed, restored: false });
        expect(screen.queryByText("Your Store version was put back.")).toBeNull();
    });

    it("marks the failed step for assistive technology", () => {
        render(InstallProgressScreen, failed);
        expect(screen.getByText("Failed")).toHaveClass("amml-visually-hidden");
    });
});

describe("FirstRunChecksScreen", () => {
    it("holds Next back when asked and keeps Skip on", async () => {
        const onnext = vi.fn();
        const onskip = vi.fn();
        render(FirstRunChecksScreen, { checks: SetupFixtures.checks, text: firstRunChecksEn, nextDisabled: true, onnext, onskip });

        const next = screen.getByRole("button", { name: "Next" });
        expect(next).toBeDisabled();
        await fireEvent.click(next);
        await fireEvent.click(screen.getByRole("button", { name: "Skip" }));

        expect(onnext).not.toHaveBeenCalled();
        expect(onskip).toHaveBeenCalledOnce();
    });

    it("lets Next through by default", async () => {
        const onnext = vi.fn();
        render(FirstRunChecksScreen, { checks: SetupFixtures.checks, text: firstRunChecksEn, onnext });

        await fireEvent.click(screen.getByRole("button", { name: "Next" }));

        expect(onnext).toHaveBeenCalledOnce();
    });
});
