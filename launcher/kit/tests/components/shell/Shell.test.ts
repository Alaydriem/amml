import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import SettingsNav from "../../../src/components/shell/SettingsNav.svelte";
import StatusBar from "../../../src/components/shell/StatusBar.svelte";
import TitleBar from "../../../src/components/shell/TitleBar.svelte";
import WizardFooter from "../../../src/components/shell/WizardFooter.svelte";
import { shellEn } from "../../../src/text/en/shell";

describe("TitleBar", () => {
    it("fills the registry template and names the two icon buttons", () => {
        render(TitleBar, { text: shellEn.titleBar, registryFetched: "4 min ago" });

        expect(screen.getByText("Registry · 4 min ago")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Refresh registries" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Settings" })).toBeInTheDocument();
    });

    it("shows the step caption and no buttons during setup", () => {
        render(TitleBar, { text: shellEn.titleBar, steps: { total: 3, current: 2 } });

        expect(screen.getByText("Setup · 2 of 3")).toBeInTheDocument();
        expect(screen.queryByRole("button")).toBeNull();
    });

    it("calls the callbacks of the two buttons", async () => {
        const onrefresh = vi.fn();
        const onsettings = vi.fn();
        render(TitleBar, { text: shellEn.titleBar, onrefresh, onsettings });

        await fireEvent.click(screen.getByRole("button", { name: "Refresh registries" }));
        await fireEvent.click(screen.getByRole("button", { name: "Settings" }));

        expect(onrefresh).toHaveBeenCalledOnce();
        expect(onsettings).toHaveBeenCalledOnce();
    });
});

describe("StatusBar", () => {
    it("fills both templates", () => {
        render(StatusBar, { text: shellEn.statusBar, version: "0.1.0", dataPath: "C:\\data", storeVersion: "1.26.60.5" });

        expect(screen.getByText("launcher 0.1.0 · data C:\\data")).toBeInTheDocument();
        expect(screen.getByText("Store Bedrock 1.26.60.5 registered")).toBeInTheDocument();
    });

    it("replaces the left text with a message and hides the store status without a version", () => {
        render(StatusBar, { text: shellEn.statusBar, version: "0.1.0", dataPath: "C:\\data", message: "Copied" });

        expect(screen.getByText("Copied")).toBeInTheDocument();
        expect(screen.queryByText(/Store Bedrock/)).toBeNull();
    });
});

describe("SettingsNav", () => {
    it("is a named navigation and marks the open section", () => {
        render(SettingsNav, { text: shellEn.settingsNav, items: shellEn.settingsSections, value: "downloads" });

        expect(screen.getByRole("navigation", { name: "Settings sections" })).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Downloads" })).toHaveAttribute("aria-current", "page");
        expect(screen.getByRole("button", { name: "About" })).not.toHaveAttribute("aria-current");
    });

    it("asks for a section and keeps the open one until the parent answers", async () => {
        const onchange = vi.fn();
        render(SettingsNav, { text: shellEn.settingsNav, items: shellEn.settingsSections, value: "registries", onchange });

        await fireEvent.click(screen.getByRole("button", { name: "About" }));

        expect(onchange).toHaveBeenCalledExactlyOnceWith("about");
        expect(screen.getByRole("button", { name: "Registries" })).toHaveAttribute("aria-current", "page");
    });
});

describe("WizardFooter", () => {
    it("draws the note and both buttons from its text and holds the primary back when asked", async () => {
        const onsecondary = vi.fn();
        render(WizardFooter, {
            text: { note: "May ask for admin", secondary: "Skip", primary: "Next" },
            primaryDisabled: true,
            onsecondary,
        });

        expect(screen.getByText("May ask for admin")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Next" })).toBeDisabled();
        await fireEvent.click(screen.getByRole("button", { name: "Skip" }));
        expect(onsecondary).toHaveBeenCalledOnce();
    });
});
