import { describe, expect, it, vi } from "vitest";
import { ApiFault } from "#lib/api/ApiFault.ts";
import type { FaultKindView } from "#lib/bindings/FaultKindView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import { FaultActions } from "#lib/stores/FaultActions.ts";

function fault(kind: FaultKindView, code: string | null = null): FaultView {
    return { kind, code, values: {}, restored: false };
}

function actions() {
    const shell = { showLog: vi.fn(async () => {}), dismissStartupFault: vi.fn(), say: vi.fn() };
    const api = { openUrl: vi.fn(async () => {}) };
    const navigate = vi.fn();
    const copy = vi.fn(async () => {});
    return { shell, api, navigate, copy, run: new FaultActions(shell, api, navigate, copy) };
}

describe("FaultActions", () => {
    it("sends a missing app to its Store page", async () => {
        const { api, run } = actions();

        await run.run(fault("store-missing"));
        await run.run(fault("gaming-services-missing"));
        await run.run(fault("preview-missing"));

        expect((api.openUrl.mock.calls as unknown as string[][]).map((call) => call[0])).toEqual([
            "ms-windows-store://pdp/?ProductId=9NBLGGH2JHXJ",
            "ms-windows-store://pdp/?ProductId=9MWPM2CQNLHN",
            "ms-windows-store://pdp/?ProductId=9P5X4QVLC2XR",
        ]);
    });

    it("shows the log for a general fault and for the cut-off install, which then goes away", async () => {
        const { shell, run } = actions();

        await run.run(fault("operation-failed"));
        expect(shell.dismissStartupFault).not.toHaveBeenCalled();
        await run.run(fault("install-interrupted"));

        expect(shell.showLog).toHaveBeenCalledTimes(2);
        expect(shell.dismissStartupFault).toHaveBeenCalledOnce();
    });

    it("opens the progress for a busy launcher and copies the code of a refused install", async () => {
        const { shell, navigate, copy, run } = actions();

        await run.run(fault("busy"));
        await run.run(fault("deploy-refused", "0x80073CF3"));
        await run.run(fault("restore-failed", String.raw`C:\amml\original.msixvc`));

        expect(navigate).toHaveBeenCalledWith("/instances/install");
        expect(copy).toHaveBeenCalledWith("0x80073CF3");
        expect(copy).toHaveBeenCalledWith(String.raw`C:\amml\original.msixvc`);
        expect(shell.say).toHaveBeenCalledWith("Copied");
    });

    it("tries again for the faults whose action says so", async () => {
        const { run } = actions();
        const retry = vi.fn();

        for (const kind of ["download-failed", "copy-failed", "disk-space", "instance-running"] as const) {
            await run.run(fault(kind), retry);
        }
        await run.run(fault("consent-needed"), retry);

        expect(retry).toHaveBeenCalledTimes(4);
    });
});

describe("ApiFault", () => {
    it("keeps the fault the bridge sent and turns any other error into a general fault", () => {
        const sent = fault("busy");
        expect(ApiFault.of(sent)).toBe(sent);
        expect(ApiFault.of(new ApiFault(sent))).toBe(sent);
        expect(ApiFault.of(new Error("boom"))).toEqual({ kind: "operation-failed", code: "boom", values: {}, restored: false });
        expect(ApiFault.of("text").code).toBe("text");
    });
});
