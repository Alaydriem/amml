import type { RegistriesSettingsScreenText } from "../../screens/RegistriesSettingsScreen";

export const registriesSettingsEn: RegistriesSettingsScreenText = {
    heading: "Registries",
    add: "Add a registry URL",
    row: {
        builtIn: "Built in",
        remove: "Remove",
    },
    statusOk: { one: "{count} mod · {fetched}", other: "{count} mods · {fetched}" },
    statusOkNoCount: "{fetched}",
    statusFailed: "Fetch failed",
    statusOffline: "Offline",
    statusLoading: "Checking",
    note: "A registry is a list of mods and AMML versions, with a checksum for each download. The launcher reads them at start and when you press refresh. Mods from other registries show where they came from.",
};
