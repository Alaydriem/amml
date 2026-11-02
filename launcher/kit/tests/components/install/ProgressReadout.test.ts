import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import InstallStepRow from "../../../src/components/install/InstallStepRow.svelte";
import ProgressReadout from "../../../src/components/install/ProgressReadout.svelte";
import { ProgressReadoutRules } from "../../../src/components/install/ProgressReadout";
import { InstallFixtures } from "../../../src/fixtures/install";
import { installProgressEn } from "../../../src/text/en/installProgress";

const text = installProgressEn.readout;

describe("ProgressReadout", () => {
    it("shows the figures line, the detail and the bar", () => {
        render(ProgressReadout, { eyebrow: "Step 3 of 6", progress: InstallFixtures.progress, text });
        expect(screen.getByText("1.28 / 2.06 GB · 38 MB/s · about 20 s left")).toBeInTheDocument();
        expect(screen.getByText(/mirror 1 of 2/)).toBeInTheDocument();
        expect(screen.getByRole("progressbar", { name: "Download progress" })).toBeInTheDocument();
    });

    it("leaves out the rate and the time left while they are unknown", () => {
        expect(ProgressReadoutRules.line({ done: 5_000_000, total: 50_000_000 }, text)).toBe("5.0 / 50.0 MB");
    });

    it("joins the figures with the separator of the text", () => {
        const line = ProgressReadoutRules.line(InstallFixtures.progress, { ...text, separator: " | " });
        expect(line).toBe("1.28 / 2.06 GB | 38 MB/s | about 20 s left");
    });
});

describe("InstallStepRow", () => {
    const steps = installProgressEn.steps;

    it("names each step state for assistive technology", () => {
        const labels = InstallFixtures.failedSteps.map((step) => {
            const { container, unmount } = render(InstallStepRow, { step, text: steps });
            const hidden = container.querySelector(".amml-visually-hidden")?.textContent;
            unmount();
            return hidden;
        });
        expect(labels).toEqual(["Done", "Done", "Failed", "Waiting", "Waiting"]);
    });

    it("marks the active step as the current one", () => {
        render(InstallStepRow, { step: InstallFixtures.steps[2]!, text: steps });
        expect(screen.getByText("Download 1.26.51.1").closest("[aria-current]")).toHaveAttribute("aria-current", "step");
    });
});
