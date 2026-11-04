import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of the registries settings page, with every string through I18n. Without a pack it equals the kit English.
export class RegistriesSettingsTextSet {
    static build(): LauncherText["registriesSettings"] {
        return {
            heading: I18n.t("Registries"),
            add: I18n.t("Add a registry URL"),
            row: {
                builtIn: I18n.t("Built in"),
                remove: I18n.t("Remove"),
            },
            statusOk: {
                one: I18n.tn("{count} mod · {fetched}", "{count} mods · {fetched}", 1),
                other: I18n.tn("{count} mod · {fetched}", "{count} mods · {fetched}", 2),
            },
            statusOkNoCount: I18n.t("{fetched}"),
            statusFailed: I18n.t("Fetch failed"),
            statusOffline: I18n.t("Offline"),
            statusLoading: I18n.t("Checking"),
            note: I18n.t("A registry is a list of mods and AMML versions, with a checksum for each download. The launcher reads them at start and when you press refresh. Mods from other registries show where they came from."),
        };
    }
}
