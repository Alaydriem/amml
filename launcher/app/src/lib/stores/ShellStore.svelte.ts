import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { ActivityTextSet } from "#lib/text/ActivityTextSet.ts";
import { ShellTextSet } from "#lib/text/ShellTextSet.ts";

export type ShellApi = Pick<
    typeof LauncherApi,
    "registriesRefresh" | "settingsGet" | "aboutGet" | "checksRun" | "startupFault" | "setupGet" | "openPath"
>;

// What the title bar, the status bar and the start-up banner show, for every route.
export class ShellStore {
    static readonly KEY = Symbol("amml-shell");
    // how long a status bar message stays
    static readonly MESSAGE_MILLIS = 3000;

    official = $state<RegistryView | null>(null);
    // every registry as the last refresh saw it
    registries = $state<RegistryView[]>([]);
    launcherVersion = $state("");
    dataPath = $state("");
    logsDir = $state("");
    storeVersion = $state<string | undefined>(undefined);
    // a cut-off install the launcher finished at start; shown on the instance list until the player presses its action
    startupFault = $state<FaultView | null>(null);
    message = $state<string | undefined>(undefined);
    refreshing = $state(false);

    readonly texts = $derived(ShellTextSet.build());

    private readonly api: ShellApi;
    private readonly clock: () => number;
    private messageTimer: ReturnType<typeof setTimeout> | undefined;

    constructor(api: ShellApi, clock: () => number = Date.now) {
        this.api = api;
        this.clock = clock;
    }

    // "/setup/checks" until the settings exist, else "/instances"
    async landing(): Promise<string> {
        const setup = await this.api.setupGet();
        return setup.needed ? "/setup/checks" : "/instances";
    }

    // Loads what the bars show. A part that fails stays empty; the routes still work.
    async start(): Promise<void> {
        await Promise.all([this.loadSettings(), this.loadVersions(), this.loadStartupFault()]);
        void this.refresh();
    }

    // asks every registry again; the title bar shows how long ago the official one answered
    async refresh(): Promise<void> {
        this.refreshing = true;
        try {
            const views = await this.api.registriesRefresh();
            this.registries = views;
            this.official = views.find((view) => view.official) ?? null;
        } catch {
            // a busy launcher or a dead network leaves the last answer on screen
        } finally {
            this.refreshing = false;
        }
    }

    // the age of the official registry's answer, or nothing while it has none
    registryFetched(): string | undefined {
        const fetched = this.official?.fetched;
        return fetched === null || fetched === undefined ? undefined : ActivityTextSet.ago(this.clock() - fetched);
    }

    dismissStartupFault(): void {
        this.startupFault = null;
    }

    async showLog(): Promise<void> {
        if (this.logsDir !== "") await this.api.openPath(this.logsDir);
    }

    // shows `text` in the status bar for a moment
    say(text: string): void {
        this.message = text;
        if (this.messageTimer !== undefined) clearTimeout(this.messageTimer);
        this.messageTimer = setTimeout(() => {
            this.message = undefined;
            this.messageTimer = undefined;
        }, ShellStore.MESSAGE_MILLIS);
    }

    dispose(): void {
        if (this.messageTimer !== undefined) clearTimeout(this.messageTimer);
    }

    private async loadSettings(): Promise<void> {
        try {
            const settings = await this.api.settingsGet();
            this.dataPath = settings.dataDir;
            this.logsDir = settings.logsDir;
        } catch {
            // the bar stays empty
        }
    }

    private async loadVersions(): Promise<void> {
        try {
            this.launcherVersion = (await this.api.aboutGet()).launcherVersion;
            const store = (await this.api.checksRun()).find((check) => check.id === "store");
            this.storeVersion = store?.state === "ok" ? (store.version ?? undefined) : undefined;
        } catch {
            // the bar stays empty
        }
    }

    // The recovery runs on its own thread at start, so the answer may come after the first look; the instance list asks again.
    async loadStartupFault(): Promise<void> {
        try {
            this.startupFault = await this.api.startupFault();
        } catch {
            this.startupFault = null;
        }
    }
}
