import type { FaultView } from "#lib/bindings/FaultView.ts";

// What a failed command throws: the fault the bridge sent, or one made from any other error.
export class ApiFault extends Error {
    readonly fault: FaultView;

    constructor(fault: FaultView) {
        super(fault.code ?? fault.kind);
        this.fault = fault;
    }

    // the fault inside a thrown value
    static of(error: unknown): FaultView {
        if (error instanceof ApiFault) return error.fault;
        if (ApiFault.isFault(error)) return error;
        const text = error instanceof Error ? error.message : String(error);
        return { kind: "operation-failed", code: text, values: {}, restored: false };
    }

    private static isFault(value: unknown): value is FaultView {
        if (value === null || typeof value !== "object") return false;
        const record = value as Record<string, unknown>;
        return typeof record.kind === "string" && typeof record.values === "object";
    }
}
