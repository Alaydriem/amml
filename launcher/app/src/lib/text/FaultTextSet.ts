import type { FaultKindView } from "#lib/bindings/FaultKindView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { Units, type LauncherFaultText } from "@amml/launcher-kit";
import { UnitsTextSet } from "./UnitsTextSet.ts";

// The words of each named fault. The kit has the wording of the kinds without figures; the figures of the others
// (sizes, counts, versions) come from the fault's values, which the bridge sends as plain text.
export class FaultTextSet {
    static build(kind: FaultKindView, values: Record<string, string>): LauncherFaultText {
        const units = new Units(UnitsTextSet.build());
        switch (kind) {
            case "store-missing":
                return {
                    title: I18n.t("Minecraft is not installed from the Store."),
                    body: I18n.t(
                        "The launcher needs your Store copy to install versions. Install it from the Microsoft Store or the Xbox app, then check again.",
                    ),
                    action: I18n.t("Open the Store"),
                };
            case "gaming-services-missing":
                return {
                    title: I18n.t("Gaming Services is missing."),
                    body: I18n.t("Windows needs it to install game versions. It is free in the Microsoft Store."),
                    action: I18n.t("Open the Store"),
                };
            case "preview-missing":
                return {
                    title: I18n.t("Minecraft Preview is not installed."),
                    body: I18n.t("Preview builds install from the Preview app's licence."),
                    action: I18n.t("Open the Store"),
                };
            case "download-failed":
                return {
                    title: I18n.t("Download failed on both mirrors."),
                    body: FaultTextSet.kept(units, values),
                    action: I18n.t("Try again"),
                };
            case "deploy-refused":
                return {
                    title: I18n.tf("Windows refused to install {version}.", { version: values.version ?? "" }),
                    body: I18n.t("Your Store version was put back and starts as before."),
                    action: I18n.t("Copy details"),
                };
            case "copy-failed":
                return {
                    title:
                        values.copied === undefined || values.expected === undefined
                            ? I18n.t("Copying the game files failed.")
                            : I18n.tf("Copy failed at {copied} of {expected} files.", {
                                  copied: FaultTextSet.count(units, values.copied),
                                  expected: FaultTextSet.count(units, values.expected),
                              }),
                    body: I18n.t("No instance folder was made. Your Store version was put back."),
                    action: I18n.t("Try again"),
                };
            case "disk-space":
                return {
                    title: I18n.tf("Not enough space on {drive}.", { drive: values.drive ?? "" }),
                    body: I18n.tf(
                        "This install needs {needed} at once for the download, the copy and the instance. {free} is free.",
                        {
                            needed: FaultTextSet.size(units, values.needed),
                            free: FaultTextSet.size(units, values.free),
                        },
                    ),
                    action: I18n.t("Check again"),
                };
            case "install-interrupted":
                return {
                    title: I18n.t("The last install was cut off."),
                    body: I18n.tf(
                        "The launcher finished putting your Store version {version} back. It starts as before.",
                        { version: values.version ?? "" },
                    ),
                    action: I18n.t("Show log"),
                };
            case "restore-failed":
                return {
                    title: I18n.t("Your Store version was not put back."),
                    body: I18n.t(
                        "Put it back with the command in the details, or reinstall Minecraft from the Store.",
                    ),
                    action: I18n.t("Copy details"),
                };
            case "operation-failed":
                return {
                    title: I18n.t("That did not work."),
                    body: I18n.t("The details are below."),
                    action: I18n.t("Show log"),
                };
            case "busy":
                return {
                    title: I18n.t("Another task is running."),
                    body: I18n.t("Wait for it to end, then try again."),
                    action: I18n.t("Show progress"),
                };
            case "instance-running":
                return {
                    title: I18n.tf("{name} is running.", { name: values.name ?? "" }),
                    body: I18n.t("Close the game first. The launcher ends only games it started."),
                    action: I18n.t("Try again"),
                };
            case "consent-needed":
                // this fault opens the Developer Mode dialog and never shows a banner
                return { title: "", body: "", action: "" };
        }
    }

    // "{done} of {total} is kept ..." when the bridge knows the figures, else the same promise without them
    private static kept(units: Units, values: Record<string, string>): string {
        if (values.done === undefined || values.total === undefined) {
            return I18n.t("What came down is kept and the download continues from there.");
        }
        return I18n.tf("{done} of {total} is kept and the download continues from there.", {
            done: FaultTextSet.size(units, values.done),
            total: FaultTextSet.size(units, values.total),
        });
    }

    private static size(units: Units, bytes: string | undefined): string {
        return units.size(Number(bytes ?? 0), { digits: 1 });
    }

    private static count(units: Units, text: string | undefined): string {
        return units.count(Number(text ?? 0));
    }
}
