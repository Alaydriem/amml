import type { AddModDialogText, ConfirmDialogText } from "../../components/dialogs";

/**
 * The text of the three dialogs of artboard 08. The caller fills `{name}`, `{version}` and `{file}` through the
 * `values` prop of the dialog.
 */
export interface DialogsText {
    developerMode: ConfirmDialogText;
    deleteInstance: ConfirmDialogText;
    addMod: AddModDialogText;
}

export const dialogsEn: DialogsText = {
    developerMode: {
        eyebrow: "Needs your OK",
        title: "Turn on Developer Mode?",
        body: "Windows would not let the launcher copy **{file}** for {version}. With Developer Mode on, it can. Windows asks for admin rights once.",
        note: "If you say no, the install stops and your Store version is put back.",
        cancel: "Stop install",
        confirm: "Turn it on",
    },
    deleteInstance: {
        title: "Delete {name}?",
        body: "This deletes the folder below, with its AMML, mods and mod settings.",
        note: "This instance keeps its own worlds in that folder. They are deleted too.",
        cancel: "Keep it",
        confirm: "Delete folder",
    },
    addMod: {
        title: "Add a mod to {name}",
        fieldLabel: "Registry id or zip URL",
        unofficial:
            "**Not in any registry.** It installs as unofficial: no checksum, and the launcher never checks it for updates. Only add mods from people you trust.",
        suggestionsLabel: "Or from the registry",
        add: "Add",
        cancel: "Cancel",
        install: "Install unofficial",
    },
};
