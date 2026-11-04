import type { AboutView } from "#lib/bindings/AboutView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { InstancePresenter } from "#lib/present/InstancePresenter.ts";
import type { BugReportLine } from "@amml/launcher-kit";
import { ActivityTextSet } from "./ActivityTextSet.ts";

// The environment lines of the bug report on the About page.
export class ReportTextSet {
    static lines(about: AboutView, now: number): BugReportLine[] {
        return [
            {
                id: "windows",
                value:
                    about.windowsVersion === null
                        ? I18n.t("unknown")
                        : I18n.tf("Windows {version}", { version: about.windowsVersion }),
            },
            { id: "store", value: about.storeVersion ?? I18n.t("not installed") },
            { id: "devMode", value: about.developerMode ? I18n.t("On") : I18n.t("Off") },
            { id: "registry", value: ReportTextSet.registry(about, now) },
            { id: "logs", value: InstancePresenter.shownFolder(about.logsDir), href: about.logsDir },
        ];
    }

    private static registry(about: AboutView, now: number): string {
        const registry = about.registry;
        if (registry === null) return I18n.t("not checked yet");
        const status = ReportTextSet.status(registry.status);
        return registry.fetched === null
            ? status
            : I18n.tf("{status}, {ago}", { status, ago: ActivityTextSet.ago(now - registry.fetched) });
    }

    private static status(status: NonNullable<AboutView["registry"]>["status"]): string {
        switch (status) {
            case "ok":
                return I18n.t("ok");
            case "stale":
                return I18n.t("saved copy");
            case "offline":
                return I18n.t("offline");
            case "broken":
                return I18n.t("failed");
            case "loading":
                return I18n.t("checking");
        }
    }
}
