import type { FaultKindView } from "#lib/bindings/FaultKindView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import { FaultTextSet } from "#lib/text/FaultTextSet.ts";
import type { LauncherFault, LauncherFaultKind, LauncherFaultText } from "@amml/launcher-kit";

// What a `FaultBanner` needs for a fault, and the bridge's own kind for the code that handles the action.
export interface PresentedFault {
    kind: FaultKindView;
    fault: LauncherFault;
    text: LauncherFaultText;
}

// Turns a fault's facts into the kit's `LauncherFault` and its words.
export class FaultPresenter {
    private static readonly WARN: readonly FaultKindView[] = ["preview-missing", "disk-space", "busy", "instance-running"];
    private static readonly PRIMARY: readonly FaultKindView[] = ["download-failed", "copy-failed"];

    static present(view: FaultView): PresentedFault {
        return {
            kind: view.kind,
            fault: {
                kind: FaultPresenter.kitKind(view.kind),
                severity: FaultPresenter.severity(view.kind),
                code: view.code ?? undefined,
                primaryAction: FaultPresenter.PRIMARY.includes(view.kind),
            },
            text: FaultTextSet.build(view.kind, view.values),
        };
    }

    // The kit has nine kinds. The three the app raises itself show as the general one; the app's own kind decides
    // what the action does.
    private static kitKind(kind: FaultKindView): LauncherFaultKind {
        switch (kind) {
            case "busy":
            case "instance-running":
            case "consent-needed":
                return "operation-failed";
            default:
                return kind;
        }
    }

    private static severity(kind: FaultKindView): LauncherFault["severity"] {
        if (kind === "install-interrupted") return "info";
        return FaultPresenter.WARN.includes(kind) ? "warn" : "fault";
    }
}
