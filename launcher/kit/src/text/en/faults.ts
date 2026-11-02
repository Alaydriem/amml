import type { LauncherFaultKind, LauncherFaultText } from "../../model";

/** The eight named error states of artboard 08, with the Paper wording. The app fills the figures and codes. */
export const faultsEn: Record<LauncherFaultKind, LauncherFaultText> = {
    "store-missing": {
        title: "Minecraft is not installed from the Store.",
        body: "The launcher needs your Store copy to install versions. Install it from the Microsoft Store or the Xbox app, then check again.",
        action: "Open the Store",
    },
    "gaming-services-missing": {
        title: "Gaming Services is missing.",
        body: "Windows needs it to install game versions. It is free in the Microsoft Store.",
        action: "Open the Store",
    },
    "preview-missing": {
        title: "Minecraft Preview is not installed.",
        body: "Preview builds install from the Preview app's licence.",
        action: "Open the Store",
    },
    "download-failed": {
        title: "Download failed on both mirrors.",
        body: "1.28 of 2.06 GB is kept and the download continues from there.",
        action: "Try again",
    },
    "deploy-refused": {
        title: "Windows refused to install 1.26.51.1.",
        body: "Your Store version was put back and starts as before.",
        action: "Copy details",
    },
    "copy-failed": {
        title: "Copy failed at 9,412 of 12,178 files.",
        body: "No instance folder was made. Your Store version was put back.",
        action: "Try again",
    },
    "disk-space": {
        title: "Not enough space on C:\\.",
        body: "This install needs 6.1 GB at once for the download, the copy and the instance. 3.4 GB is free.",
        action: "Check again",
    },
    "install-interrupted": {
        title: "The last install was cut off.",
        body: "The launcher finished putting your Store version 1.26.60.5 back. It starts as before.",
        action: "Show log",
    },
};
