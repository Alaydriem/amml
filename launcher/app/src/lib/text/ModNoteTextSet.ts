import { I18n } from "#lib/i18n/index.ts";

// The words the app adds to a mod row that the kit does not hold: the loader's name and the short notes.
export interface ModNoteText {
    loaderTitle: string;
    coreAlwaysOn: string;
    official: string;
    local: string;
    fromUrl: string;
    // template with an `{api}` slot
    api: string;
    // template with a `{version}` slot
    needsAmml: string;
    // template with `{version}` and `{game}` slots
    needsAmmlGame: string;
    // template with a `{version}` slot
    laterNeedsAmml: string;
}

export class ModNoteTextSet {
    static build(): ModNoteText {
        return {
            loaderTitle: I18n.t("AMML loader"),
            coreAlwaysOn: I18n.t("core, always on"),
            official: I18n.t("official"),
            local: I18n.t("local"),
            fromUrl: I18n.t("from a URL"),
            api: I18n.t("api {api}"),
            needsAmml: I18n.t("Needs AMML {version}"),
            needsAmmlGame: I18n.t("Needs AMML {version} (game {game})"),
            laterNeedsAmml: I18n.t("Later release needs AMML {version}"),
        };
    }
}
