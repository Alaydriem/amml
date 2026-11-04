import type { UrlCheckView } from "#lib/bindings/UrlCheckView.ts";
import { I18n } from "#lib/i18n/index.ts";
import type { FoundMod, UrlCheck } from "@amml/launcher-kit";

// The rows of the URL install checks, and the mod they found.
export class UrlCheckTextSet {
    // the checks of a link that has been looked at; none before the first look
    static rows(url: string, check: UrlCheckView | null): UrlCheck[] {
        if (check === null) return [];
        const secure = url.trim().toLowerCase().startsWith("https://");
        return [
            {
                id: "https",
                title: I18n.t("The link uses https"),
                severity: secure ? "ok" : "warn",
                value: secure ? "https" : "http",
            },
            { id: "zip", title: I18n.t("Downloaded the zip"), severity: "ok" },
            {
                id: "folder",
                title: I18n.t("Holds one mod, and its amml.hcl matches the folder"),
                severity: "ok",
                value: check.id,
                valueTone: "dim",
            },
            {
                id: "api",
                title: I18n.t("Works with this instance's AMML"),
                severity: check.apiMatches ? "ok" : "warn",
                value: I18n.tf("api {api}", { api: check.api }),
                valueTone: check.apiMatches ? "ok" : "warn",
            },
            check.matchedRegistry === null
                ? {
                      id: "registry",
                      title: I18n.t("Not in any of your registries"),
                      severity: "warn",
                      value: I18n.t("no checksum"),
                  }
                : {
                      id: "registry",
                      title: I18n.t("Listed in one of your registries"),
                      severity: "ok",
                      value: I18n.t("checksum"),
                  },
        ];
    }

    // the mod the checks found, once the zip is a mod that fits the instance
    static found(check: UrlCheckView | null): FoundMod | undefined {
        if (check === null || !check.apiMatches) return undefined;
        const label = check.title ?? check.id;
        const shown = check.version === null ? label : I18n.tf("{title} {version}", { title: label, version: check.version });
        return { label: shown, id: check.id };
    }
}
