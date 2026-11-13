import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { AboutView } from "#lib/bindings/AboutView.ts";
import type { DataOverviewView } from "#lib/bindings/DataOverviewView.ts";
import type { DownloadsView } from "#lib/bindings/DownloadsView.ts";
import type { LauncherUpdateView } from "#lib/bindings/LauncherUpdateView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import type { SettingsView } from "#lib/bindings/SettingsView.ts";
import type { VersionView } from "#lib/bindings/VersionView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { DataPresenter } from "#lib/present/DataPresenter.ts";
import { DownloadsPresenter } from "#lib/present/DownloadsPresenter.ts";
import { InstancePresenter } from "#lib/present/InstancePresenter.ts";
import { RegistryPresenter } from "#lib/present/RegistryPresenter.ts";
import { VersionPresenter } from "#lib/present/VersionPresenter.ts";
import { AboutSettingsTextSet } from "#lib/text/AboutSettingsTextSet.ts";
import { DialogsTextSet } from "#lib/text/DialogsTextSet.ts";
import { DownloadsSettingsTextSet } from "#lib/text/DownloadsSettingsTextSet.ts";
import { FaultTextSet } from "#lib/text/FaultTextSet.ts";
import { GameVersionsSettingsTextSet } from "#lib/text/GameVersionsSettingsTextSet.ts";
import { RegistriesSettingsTextSet } from "#lib/text/RegistriesSettingsTextSet.ts";
import { ReportTextSet } from "#lib/text/ReportTextSet.ts";
import { SharedDataSettingsTextSet } from "#lib/text/SharedDataSettingsTextSet.ts";
import { ShellTextSet } from "#lib/text/ShellTextSet.ts";
import {
    type AboutAuthor,
    type AboutLink,
    type BugReportLine,
    type DownloadFile,
    type GameVersion,
    type Option,
    type RegistrySource,
    type SharedDataStats,
    type WorldLocation,
    type WorldSource,
} from "@amml/launcher-kit";
import { RegistryFeed, type RegistryFeedEvents } from "./RegistryFeed.ts";
import type { Navigate } from "./Navigate.ts";

export type SettingsApi = Pick<
    typeof LauncherApi,
    | "settingsGet"
    | "settingsSetPreview"
    | "registriesGet"
    | "registriesRefresh"
    | "registryAdd"
    | "registryRemove"
    | "versionsOnDisk"
    | "versionsList"
    | "versionRemove"
    | "dataOverview"
    | "worldsCopy"
    | "downloadsGet"
    | "downloadRemove"
    | "downloadsRemoveUnfinished"
    | "aboutGet"
    | "updateCheck"
    | "updateInstall"
    | "openPath"
    | "openUrl"
>;

export type SettingsSection = "registries" | "versions" | "shared" | "downloads" | "about";

// The five settings pages and what they do.
export class SettingsStore {
    static readonly SECTIONS: readonly SettingsSection[] = ["registries", "versions", "shared", "downloads", "about"];
    static readonly VERSION_SOURCE = "github.com/MinecraftBedrockArchiver/GdkLinks";
    static readonly AUTHOR: AboutAuthor = {
        name: "Alaydriem",
        initials: "AL",
        link: "github.com/Alaydriem",
        url: "https://github.com/Alaydriem",
    };
    static readonly LINKS: AboutLink[] = [
        { id: "source", href: "https://github.com/alaydriem/amml" },
        { id: "issues", href: "https://github.com/alaydriem/amml/issues" },
        { id: "licence", href: "https://github.com/alaydriem/amml/blob/master/LICENSE" },
        { id: "notices", href: "https://github.com/alaydriem/amml/blob/master/launcher/app/NOTICES.md" },
    ];

    section = $state<SettingsSection>("registries");
    settings = $state<SettingsView | null>(null);
    registries = $state<RegistryView[]>([]);
    // the add-a-registry dialog
    addOpen = $state(false);
    addValue = $state("");
    addProblem = $state<string | undefined>(undefined);
    installed = $state<VersionView[]>([]);
    releaseCount = $state(0);
    previewCount = $state(0);
    versionsFetchedAt = $state<number | null>(null);
    pendingVersion = $state<string | null>(null);
    overview = $state<DataOverviewView | null>(null);
    downloads = $state<DownloadsView | null>(null);
    about = $state<AboutView | null>(null);
    update = $state<LauncherUpdateView | null>(null);
    fault = $state<FaultView | null>(null);

    readonly texts = $derived({
        shell: ShellTextSet.build(),
        registries: RegistriesSettingsTextSet.build(),
        versions: GameVersionsSettingsTextSet.build(),
        shared: SharedDataSettingsTextSet.build(),
        downloads: DownloadsSettingsTextSet.build(),
        about: AboutSettingsTextSet.build(),
        addRegistry: DialogsTextSet.build().addRegistry,
        removeVersion: DialogsTextSet.removeVersion(),
    });

    private readonly api: SettingsApi;
    private readonly navigate: Navigate;
    private readonly clipboard: (text: string) => Promise<void>;
    private readonly say: (text: string) => void;
    private readonly clock: () => number;
    private readonly feed: RegistryFeed;

    constructor(
        api: SettingsApi,
        navigate: Navigate,
        clipboard: (text: string) => Promise<void>,
        say: (text: string) => void,
        registryEvents: RegistryFeedEvents,
        clock: () => number = Date.now,
    ) {
        this.api = api;
        this.navigate = navigate;
        this.clipboard = clipboard;
        this.say = say;
        this.clock = clock;
        this.feed = new RegistryFeed(registryEvents, (views) => {
            this.registries = views;
        });
    }

    stopWatching(): void {
        this.feed.stopWatching();
    }

    // a section id from the address; anything unknown is the registries page
    static sectionOf(id: string | undefined): SettingsSection {
        return SettingsStore.SECTIONS.find((section) => section === id) ?? "registries";
    }

    // the sections of the nav, in order
    get items(): Option[] {
        return this.texts.shell.settingsSections;
    }

    // Opens a page and reads what it shows.
    async open(id: string | undefined): Promise<void> {
        this.section = SettingsStore.sectionOf(id);
        this.fault = null;
        await this.guard(async () => {
            if (this.settings === null) this.settings = await this.api.settingsGet();
            switch (this.section) {
                case "registries":
                    await this.feed.load(() => this.api.registriesGet());
                    break;
                case "versions":
                    await this.loadVersions();
                    break;
                case "shared":
                    this.overview = await this.api.dataOverview();
                    break;
                case "downloads":
                    this.downloads = await this.api.downloadsGet();
                    break;
                case "about":
                    this.about = await this.api.aboutGet();
                    this.update = await this.checkUpdate();
                    break;
            }
        });
    }

    // the nav picked a section
    pick(id: string): void {
        this.navigate(`/settings/${id}`);
    }

    back(): void {
        this.navigate("/instances");
    }

    // ---- registries ----

    sources(): RegistrySource[] {
        const now = this.clock();
        return this.registries.map((view) => RegistryPresenter.toSource(view, now));
    }

    openAdd(): void {
        this.addOpen = true;
        this.addValue = "";
        this.addProblem = undefined;
    }

    setAddValue(value: string): void {
        this.addValue = value;
        this.addProblem = undefined;
    }

    closeAdd(): void {
        this.addOpen = false;
    }

    // A refused address keeps the dialog open and says why.
    async add(url: string): Promise<void> {
        try {
            await this.api.registryAdd(url);
            this.addOpen = false;
            this.registries = await this.api.registriesRefresh();
        } catch (error) {
            this.addProblem = SettingsStore.problemOf(ApiFault.of(error));
        }
    }

    // the row's id is the registry's cache key
    async removeRegistry(id: string): Promise<void> {
        const view = this.registries.find((held) => held.id === id);
        if (!view || view.official) return;
        await this.guard(async () => {
            await this.api.registryRemove(view.base);
            this.registries = await this.api.registriesRefresh();
        });
    }

    // ---- game versions ----

    async setPreview(on: boolean): Promise<void> {
        await this.guard(async () => {
            await this.api.settingsSetPreview(on);
            this.settings = await this.api.settingsGet();
        });
    }

    gameVersions(): GameVersion[] {
        return this.installed.map((view) => VersionPresenter.toGameVersion(view));
    }

    // the age of the version list, or a plain word before it has been read
    versionsFetched(): string {
        if (this.versionsFetchedAt === null) return I18n.t("never");
        return this.agoText(this.versionsFetchedAt);
    }

    async refreshVersions(): Promise<void> {
        await this.guard(() => this.loadVersions());
    }

    // The player asked to remove a version; nothing is removed until the confirmation.
    askRemoveVersion(version: string): void {
        this.pendingVersion = version;
    }

    cancelRemoveVersion(): void {
        this.pendingVersion = null;
    }

    async confirmRemoveVersion(): Promise<void> {
        const version = this.pendingVersion;
        if (version === null) return;
        this.pendingVersion = null;
        await this.guard(async () => {
            await this.api.versionRemove(version);
            this.installed = await this.api.versionsOnDisk();
        });
    }

    versionsFolder(): string {
        const logs = this.settings?.logsDir ?? "";
        const root = logs.replace(/\\launcher\\logs$/, "");
        return InstancePresenter.shownFolder(`${root}\\versions`);
    }

    // ---- shared data ----

    stats(): SharedDataStats | null {
        return this.overview === null ? null : DataPresenter.toStats(this.overview);
    }

    locations(): WorldLocation[] {
        return this.overview === null ? [] : DataPresenter.toLocations(this.overview);
    }

    worldSources(): WorldSource[] {
        return this.overview === null
            ? []
            : DataPresenter.toSources(this.overview.sources, { appdata: I18n.t("Store Minecraft") }, this.clock());
    }

    async openSharedFolder(): Promise<void> {
        if (this.overview) await this.guard(() => this.api.openPath(this.overview?.folder ?? ""));
    }

    // copying runs as a job; its progress is on the install screen
    async copyWorlds(id: string): Promise<void> {
        await this.guard(async () => {
            await this.api.worldsCopy(id);
            this.navigate("/instances/install");
        });
    }

    // ---- downloads ----

    downloadFiles(): DownloadFile[] {
        return this.downloads === null ? [] : DownloadsPresenter.toFiles(this.downloads, this.clock());
    }

    async removeFile(id: string): Promise<void> {
        await this.guard(async () => {
            await this.api.downloadRemove(id);
            this.downloads = await this.api.downloadsGet();
        });
    }

    async removeUnfinished(): Promise<void> {
        await this.guard(async () => {
            await this.api.downloadsRemoveUnfinished();
            this.downloads = await this.api.downloadsGet();
        });
    }

    // ---- about ----

    platform(): string {
        return I18n.t("Windows x64");
    }

    report(): BugReportLine[] {
        return this.about === null ? [] : ReportTextSet.lines(this.about, this.clock());
    }

    // Opens the author, a project link or the logs folder.
    async openLink(href: string): Promise<void> {
        await this.guard(() => (href.startsWith("https://") ? this.api.openUrl(href) : this.api.openPath(href)));
    }

    // The updater answers `null` when there is nothing newer or the check failed; a refused call is no update either.
    private async checkUpdate(): Promise<LauncherUpdateView | null> {
        try {
            return await this.api.updateCheck();
        } catch {
            return null;
        }
    }

    async installUpdate(): Promise<void> {
        await this.guard(() => this.api.updateInstall());
    }

    async copyReport(report: string): Promise<void> {
        await this.guard(async () => {
            await this.clipboard(report);
            this.say(I18n.t("Report copied"));
        });
    }

    private async loadVersions(): Promise<void> {
        this.installed = await this.api.versionsOnDisk();
        try {
            const [release, preview] = await Promise.all([this.api.versionsList("release"), this.api.versionsList("preview")]);
            this.releaseCount = release.length;
            this.previewCount = preview.length;
            this.versionsFetchedAt = this.clock();
        } catch {
            // the lists need the network and a free launcher; the installed versions still show
        }
    }

    private agoText(time: number): string {
        const seconds = Math.max(0, Math.round((this.clock() - time) / 1000));
        return seconds < 60 ? I18n.t("just now") : I18n.tf("{n} min ago", { n: Math.floor(seconds / 60) });
    }

    // the sentence a dialog shows for a refused address; the error's own text stays out of it
    private static problemOf(fault: FaultView): string {
        return FaultTextSet.build(fault.kind, fault.values).title;
    }

    private async guard(work: () => Promise<unknown>): Promise<void> {
        try {
            await work();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }
}
