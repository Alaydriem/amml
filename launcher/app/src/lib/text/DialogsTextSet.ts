import { I18n } from "#lib/i18n/index.ts";
import type { ConfirmDialogText, LauncherText } from "@amml/launcher-kit";

// The text of the dialogs, with every string through I18n. Without a pack it equals the kit English.
export class DialogsTextSet {
    static build(): LauncherText["dialogs"] {
        return {
            developerMode: {
                eyebrow: I18n.t("Needs your OK"),
                title: I18n.t("Turn on Developer Mode?"),
                body: I18n.t("Windows would not let the launcher copy **{file}** for {version}. With Developer Mode on, it can. Windows asks for admin rights once."),
                note: I18n.t("If you say no, the install stops and your Store version is put back."),
                cancel: I18n.t("Stop install"),
                confirm: I18n.t("Turn it on"),
            },
            deleteInstance: {
                title: I18n.t("Delete {name}?"),
                body: I18n.t("This deletes the folder below, with its AMML, mods and mod settings."),
                note: I18n.t("This instance keeps its own worlds in that folder. They are deleted too."),
                cancel: I18n.t("Keep it"),
                confirm: I18n.t("Delete folder"),
            },
            addMod: {
                title: I18n.t("Add a mod to {name}"),
                fieldLabel: I18n.t("Registry id or zip URL"),
                unofficial: I18n.t("**Not in any registry.** It installs as unofficial: no checksum, and the launcher never checks it for updates. Only add mods from people you trust."),
                suggestionsLabel: I18n.t("Or from the registry"),
                add: I18n.t("Add"),
                cancel: I18n.t("Cancel"),
                install: I18n.t("Install unofficial"),
            },
            addRegistry: {
                title: I18n.t("Add a registry"),
                fieldLabel: I18n.t("Registry address"),
                placeholder: I18n.t("https://example.github.io/registry/"),
                invalid: I18n.t("Use an https:// address."),
                add: I18n.t("Add"),
                cancel: I18n.t("Cancel"),
            },
        };
    }

    // The confirmation before a mod is taken out of an instance.
    static removeMod(): ConfirmDialogText {
        return {
            title: I18n.t("Remove {title}?"),
            body: I18n.t("This deletes **{id}** and its settings from this instance."),
            cancel: I18n.t("Keep it"),
            confirm: I18n.t("Remove"),
        };
    }

    // The confirmation before an installed game version is removed from the version store.
    static removeVersion(): ConfirmDialogText {
        return {
            title: I18n.t("Remove {version}?"),
            body: I18n.t(
                "The launcher removes **versions\\{version}**. Instances made from it keep working. A new instance of this version downloads it again.",
            ),
            cancel: I18n.t("Cancel"),
            confirm: I18n.t("Remove"),
        };
    }
}
