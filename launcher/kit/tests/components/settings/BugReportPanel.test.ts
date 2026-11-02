import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import { BugReportPanelRules } from "../../../src/components/settings/BugReportPanel";
import BugReportPanel from "../../../src/components/settings/BugReportPanel.svelte";
import type { BugReportLine } from "../../../src/model";
import { aboutSettingsEn } from "../../../src/text/en/aboutSettings";

const text = aboutSettingsEn.bugReport;
const lines: BugReportLine[] = [
    { id: "windows", value: "11 Pro 26300 x64" },
    { id: "store", value: "1.26.60.5 · Status Ok" },
    { id: "logs", value: "launcher\\logs", href: "launcher\\logs" },
];

describe("BugReportPanel", () => {
    it("builds one line per entry from the line template, with the plain value of a link", () => {
        expect(BugReportPanelRules.report(lines, text)).toBe(
            "Windows: 11 Pro 26300 x64\nStore Bedrock: 1.26.60.5 · Status Ok\nLogs: launcher\\logs",
        );
    });

    it("follows the line template of the text", () => {
        const dashed = { ...text, line: "{label} - {value}" };

        expect(BugReportPanelRules.report(lines.slice(0, 1), dashed)).toBe("Windows - 11 Pro 26300 x64");
    });

    it("passes the report text to oncopy when Copy all is pressed", async () => {
        const oncopy = vi.fn();
        render(BugReportPanel, { text, lines, oncopy });

        await fireEvent.click(screen.getByRole("button", { name: "Copy all" }));

        expect(oncopy).toHaveBeenCalledExactlyOnceWith(BugReportPanelRules.report(lines, text));
    });

    it("draws a link value from the link template and calls onopenlink with its href", async () => {
        const onopenlink = vi.fn();
        render(BugReportPanel, { text, lines, onopenlink });

        await fireEvent.click(screen.getByRole("button", { name: "launcher\\logs · Open" }));

        expect(onopenlink).toHaveBeenCalledExactlyOnceWith("launcher\\logs");
    });

    it("draws a plain value as text, not as a button", () => {
        render(BugReportPanel, { text, lines });

        expect(screen.getByText("11 Pro 26300 x64").closest("button")).toBeNull();
    });
});
