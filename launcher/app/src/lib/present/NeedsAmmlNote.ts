import type { NeedsAmmlView } from "#lib/bindings/NeedsAmmlView.ts";
import type { ModNoteText } from "#lib/text/ModNoteTextSet.ts";
import { Template } from "@amml/launcher-kit";

// The notes the launcher shows next to a mod that needs a newer AMML.
export class NeedsAmmlNote {
    // "Needs AMML 26.62.0 (game 1.26.70.1)", or without the game when the AMML release is not in the official root
    static blocked(needs: NeedsAmmlView, notes: ModNoteText): string {
        if (needs.game === null) return Template.fill(notes.needsAmml, { version: needs.version });
        return Template.fill(notes.needsAmmlGame, { version: needs.version, game: needs.game });
    }

    static later(needs: NeedsAmmlView, notes: ModNoteText): string {
        return Template.fill(notes.laterNeedsAmml, { version: needs.version });
    }
}
