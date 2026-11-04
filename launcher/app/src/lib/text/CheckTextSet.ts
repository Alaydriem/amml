import type { CheckIdView } from "#lib/bindings/CheckIdView.ts";
import type { CheckView } from "#lib/bindings/CheckView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { Units } from "@amml/launcher-kit";
import { UnitsTextSet } from "./UnitsTextSet.ts";

// The words of each line of the system check, from the facts of a `CheckView`.
export class CheckTextSet {
    static title(id: CheckIdView): string {
        switch (id) {
            case "store":
                return I18n.t("Minecraft from the Store");
            case "gaming-services":
                return I18n.t("Gaming Services");
            case "gameinput":
                return I18n.t("GameInput");
            case "preview":
                return I18n.t("Minecraft Preview");
            case "disk":
                return I18n.t("Disk space");
        }
    }

    static detail(check: CheckView): string {
        switch (check.id) {
            case "store":
                if (check.state === "missing") return I18n.t("Install Minecraft Bedrock from the Microsoft Store");
                return I18n.tf("{version} · {package}", {
                    version: check.version ?? "",
                    package: (check.package ?? "").split("_")[0] ?? "",
                });
            case "gaming-services":
                return I18n.t("Needed to install game versions");
            case "gameinput":
                return I18n.t("Controllers and some mods need it");
            case "preview":
                return I18n.t("Only for preview builds");
            case "disk":
                return I18n.tf("{drive} · {size} per new version", {
                    drive: check.drive ?? "",
                    size: new Units(UnitsTextSet.build()).size(CheckTextSet.VERSION_SPACE, { digits: 0 }),
                });
        }
    }

    // the right-hand text; none when the row shows an action button instead
    static value(check: CheckView): string | undefined {
        if (check.action === "install") return undefined;
        if (check.id === "disk") {
            const free = new Units(UnitsTextSet.build()).size(check.freeBytes ?? 0, { digits: 0 });
            return I18n.tf("{free} free", { free });
        }
        if (check.state === "ok") return I18n.t("Found");
        if (check.state === "warn") return I18n.t("Needs attention");
        return check.id === "preview" ? I18n.t("Not installed") : I18n.t("Not found");
    }

    static action(check: CheckView): string | undefined {
        switch (check.action) {
            case "install":
                return I18n.t("Install");
            case "open-store":
                return I18n.t("Open the Store");
            default:
                return undefined;
        }
    }

    // free space one new game version needs, as the bridge checks it
    static readonly VERSION_SPACE = 6_000_000_000;
}
