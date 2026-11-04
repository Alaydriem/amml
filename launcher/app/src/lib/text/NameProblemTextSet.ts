import type { NameProblemView } from "#lib/bindings/NameProblemView.ts";
import { I18n } from "#lib/i18n/index.ts";

// Why a new instance name cannot be used. An empty name only holds Create back, and a name that is taken is told by
// the form itself.
export class NameProblemTextSet {
    static text(problem: NameProblemView | null): string | undefined {
        if (problem !== "bad") return undefined;
        return I18n.t(
            "Use another name. A name cannot start with a dot, hold \\ / : * ? \" < > |, or be a Windows device name such as CON.",
        );
    }
}
