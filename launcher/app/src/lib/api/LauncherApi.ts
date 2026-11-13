import { invoke } from "@tauri-apps/api/core";
import type { AboutView } from "#lib/bindings/AboutView.ts";
import type { LauncherUpdateView } from "#lib/bindings/LauncherUpdateView.ts";
import type { AdoptCandidatesView } from "#lib/bindings/AdoptCandidatesView.ts";
import type { CardView } from "#lib/bindings/CardView.ts";
import type { CatalogLayoutView } from "#lib/bindings/CatalogLayoutView.ts";
import type { CatalogSortView } from "#lib/bindings/CatalogSortView.ts";
import type { ChannelView } from "#lib/bindings/ChannelView.ts";
import type { CheckView } from "#lib/bindings/CheckView.ts";
import type { DataOverviewView } from "#lib/bindings/DataOverviewView.ts";
import type { DownloadsView } from "#lib/bindings/DownloadsView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";
import type { ModDetailView } from "#lib/bindings/ModDetailView.ts";
import type { ModView } from "#lib/bindings/ModView.ts";
import type { NameProblemView } from "#lib/bindings/NameProblemView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import type { SearchGroupView } from "#lib/bindings/SearchGroupView.ts";
import type { SettingsView } from "#lib/bindings/SettingsView.ts";
import type { SetupView } from "#lib/bindings/SetupView.ts";
import type { UrlCheckView } from "#lib/bindings/UrlCheckView.ts";
import type { VersionView } from "#lib/bindings/VersionView.ts";
import type { LanguagePack } from "#lib/i18n/index.ts";
import { ApiFault } from "./ApiFault.ts";

// Every call the window makes into the launcher: one static method per Tauri command. A failed call throws an
// `ApiFault` that holds the bridge's fault.
export class LauncherApi {
    static i18nLocales(): Promise<string[]> {
        return LauncherApi.call("i18n_locales");
    }

    static i18nLoad(requested: string): Promise<LanguagePack | null> {
        return LauncherApi.call("i18n_load", { requested });
    }

    static instancesList(): Promise<InstanceView[]> {
        return LauncherApi.call("instances_list");
    }

    static instanceGet(name: string): Promise<InstanceView> {
        return LauncherApi.call("instance_get", { name });
    }

    static instanceCreate(name: string, game: string, preview: boolean, ownData: boolean): Promise<number> {
        return LauncherApi.call("instance_create", { name, game, preview, ownData });
    }

    static instanceDelete(name: string, confirm: string): Promise<void> {
        return LauncherApi.call("instance_delete", { name, confirm });
    }

    static instanceLaunch(name: string): Promise<void> {
        return LauncherApi.call("instance_launch", { name });
    }

    static modsList(name: string): Promise<ModView[]> {
        return LauncherApi.call("mods_list", { name });
    }

    static modSetEnabled(name: string, id: string, enabled: boolean): Promise<void> {
        return LauncherApi.call("mod_set_enabled", { name, id, enabled });
    }

    static modRemove(name: string, id: string): Promise<void> {
        return LauncherApi.call("mod_remove", { name, id });
    }

    static modsInstall(name: string, ids: string[]): Promise<number> {
        return LauncherApi.call("mods_install", { name, ids });
    }

    static modsUpdate(name: string, ids: string[]): Promise<number> {
        return LauncherApi.call("mods_update", { name, ids });
    }

    static ammlUpdate(name: string): Promise<number> {
        return LauncherApi.call("amml_update", { name });
    }

    static urlInspect(name: string, url: string): Promise<UrlCheckView> {
        return LauncherApi.call("url_inspect", { name, url });
    }

    static urlInstall(name: string, url: string, consent: boolean): Promise<number> {
        return LauncherApi.call("url_install", { name, url, consent });
    }

    static catalogLayout(name: string, registryId: string): Promise<CatalogLayoutView> {
        return LauncherApi.call("catalog_layout", { name, registryId });
    }

    static catalogPage(name: string, registryId: string, page: number): Promise<CardView[]> {
        return LauncherApi.call("catalog_page", { name, registryId, page });
    }

    static catalogSearch(name: string, query: string): Promise<SearchGroupView[]> {
        return LauncherApi.call("catalog_search", { name, query });
    }

    static catalogSorted(name: string, registryId: string, sort: CatalogSortView): Promise<CardView[] | null> {
        return LauncherApi.call("catalog_sorted", { name, registryId, sort });
    }

    static catalogDetail(name: string, registryId: string, id: string): Promise<ModDetailView> {
        return LauncherApi.call("catalog_detail", { name, registryId, id });
    }

    static registriesGet(): Promise<RegistryView[]> {
        return LauncherApi.call("registries_get");
    }

    static registriesRefresh(): Promise<RegistryView[]> {
        return LauncherApi.call("registries_refresh");
    }

    static registryAdd(url: string): Promise<void> {
        return LauncherApi.call("registry_add", { url });
    }

    static registryRemove(base: string): Promise<void> {
        return LauncherApi.call("registry_remove", { base });
    }

    static versionsList(channel: ChannelView): Promise<VersionView[]> {
        return LauncherApi.call("versions_list", { channel });
    }

    static versionsOnDisk(): Promise<VersionView[]> {
        return LauncherApi.call("versions_on_disk");
    }

    static versionRemove(version: string): Promise<boolean> {
        return LauncherApi.call("version_remove", { version });
    }

    static checksRun(): Promise<CheckView[]> {
        return LauncherApi.call("checks_run");
    }

    static gameInputInstall(): Promise<number> {
        return LauncherApi.call("game_input_install");
    }

    static setupGet(): Promise<SetupView> {
        return LauncherApi.call("setup_get");
    }

    static adoptCandidates(): Promise<AdoptCandidatesView> {
        return LauncherApi.call("adopt_candidates");
    }

    static setupRun(source: string): Promise<number> {
        return LauncherApi.call("setup_run", { source });
    }

    static dataOverview(): Promise<DataOverviewView> {
        return LauncherApi.call("data_overview");
    }

    static worldsCopy(source: string): Promise<number> {
        return LauncherApi.call("worlds_copy", { source });
    }

    static downloadsGet(): Promise<DownloadsView> {
        return LauncherApi.call("downloads_get");
    }

    static downloadRemove(id: string): Promise<void> {
        return LauncherApi.call("download_remove", { id });
    }

    static downloadsRemoveUnfinished(): Promise<number> {
        return LauncherApi.call("downloads_remove_unfinished");
    }

    static settingsGet(): Promise<SettingsView> {
        return LauncherApi.call("settings_get");
    }

    static settingsSetPreview(on: boolean): Promise<void> {
        return LauncherApi.call("settings_set_preview", { on });
    }

    static jobGet(): Promise<JobSnapshot | null> {
        return LauncherApi.call("job_get");
    }

    static jobCancel(): Promise<void> {
        return LauncherApi.call("job_cancel");
    }

    static jobRetry(): Promise<number> {
        return LauncherApi.call("job_retry");
    }

    static jobRetryDeveloperMode(): Promise<number> {
        return LauncherApi.call("job_retry_developer_mode");
    }

    static aboutGet(): Promise<AboutView> {
        return LauncherApi.call("about_get");
    }

    static nameProblem(name: string): Promise<NameProblemView | null> {
        return LauncherApi.call("name_problem", { name });
    }

    static startupFault(): Promise<FaultView | null> {
        return LauncherApi.call("startup_fault");
    }

    static openPath(path: string): Promise<void> {
        return LauncherApi.call("open_path", { path });
    }

    static openUrl(url: string): Promise<void> {
        return LauncherApi.call("open_url", { url });
    }

    static updateCheck(): Promise<LauncherUpdateView | null> {
        return LauncherApi.call("update_check");
    }

    static updateInstall(): Promise<void> {
        return LauncherApi.call("update_install");
    }

    private static async call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
        try {
            return await invoke<T>(command, args);
        } catch (error) {
            throw new ApiFault(ApiFault.of(error));
        }
    }
}
