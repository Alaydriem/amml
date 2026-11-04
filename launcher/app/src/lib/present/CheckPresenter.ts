import type { CheckView } from "#lib/bindings/CheckView.ts";
import { CheckTextSet } from "#lib/text/CheckTextSet.ts";
import type { SystemCheck } from "@amml/launcher-kit";

// Turns a system check's facts into the kit's `SystemCheck` model.
export class CheckPresenter {
    static toSystemCheck(view: CheckView): SystemCheck {
        return {
            id: view.id,
            title: CheckTextSet.title(view.id),
            detail: CheckTextSet.detail(view),
            severity: CheckPresenter.severity(view),
            value: CheckTextSet.value(view),
            actionLabel: CheckTextSet.action(view),
            elevated: view.elevated || undefined,
        };
    }

    // The player can go on when the Store game and Gaming Services are there.
    static canContinue(checks: readonly CheckView[]): boolean {
        return ["store", "gaming-services"].every((id) => checks.some((check) => check.id === id && check.state === "ok"));
    }

    // a missing preview app is optional; every other check that is not ok needs attention
    private static severity(view: CheckView): SystemCheck["severity"] {
        if (view.state === "ok") return "ok";
        if (view.id === "preview" && view.state === "missing") return "muted";
        return "warn";
    }
}
