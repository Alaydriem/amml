import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import FaultBanner from "../../../src/components/dialogs/FaultBanner.svelte";
import { DialogFixtures } from "../../../src/fixtures/dialogs";
import { faultsEn } from "../../../src/text/en/faults";

describe("FaultBanner", () => {
    it("calls onaction with the fault kind", async () => {
        const onaction = vi.fn();
        render(FaultBanner, { fault: DialogFixtures.faults["download-failed"], text: faultsEn["download-failed"], onaction });

        await fireEvent.click(screen.getByRole("button", { name: "Try again" }));

        expect(onaction).toHaveBeenCalledExactlyOnceWith("download-failed");
    });

    it("shows the title, the body and the code", () => {
        render(FaultBanner, { fault: DialogFixtures.faults["deploy-refused"], text: faultsEn["deploy-refused"] });

        expect(screen.getByText("Windows refused to install 1.26.51.1.")).toBeInTheDocument();
        expect(screen.getByText("0x80073CF3 · package failed updates")).toBeInTheDocument();
    });

    it("announces a fault at once and a warning politely", () => {
        const { unmount } = render(FaultBanner, { fault: DialogFixtures.faults["store-missing"], text: faultsEn["store-missing"] });
        expect(screen.getByRole("alert")).toBeInTheDocument();
        unmount();

        render(FaultBanner, { fault: DialogFixtures.faults["disk-space"], text: faultsEn["disk-space"] });
        expect(screen.getByRole("status")).toBeInTheDocument();
    });

    it("fills the button for the primary action and outlines the others", () => {
        const onaction = vi.fn();
        const { unmount } = render(FaultBanner, {
            fault: DialogFixtures.faults["download-failed"],
            text: faultsEn["download-failed"],
            onaction,
        });
        expect(screen.getByRole("button", { name: "Try again" })).toHaveClass("amml-btn--primary");
        unmount();

        render(FaultBanner, { fault: DialogFixtures.faults["store-missing"], text: faultsEn["store-missing"], onaction });
        expect(screen.getByRole("button", { name: "Open the Store" })).toHaveClass("amml-btn--secondary");
    });

    it("draws no button when nothing handles the action", () => {
        render(FaultBanner, { fault: DialogFixtures.faults["download-failed"], text: faultsEn["download-failed"] });

        expect(screen.queryByRole("button")).not.toBeInTheDocument();
    });
});
