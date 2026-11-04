import { ApiFault } from "#lib/api/ApiFault.ts";
import type { JobEvents } from "#lib/api/JobEvents.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { AdoptCandidatesView } from "#lib/bindings/AdoptCandidatesView.ts";
import type { CheckView } from "#lib/bindings/CheckView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import { CheckPresenter } from "#lib/present/CheckPresenter.ts";
import { DataPresenter } from "#lib/present/DataPresenter.ts";
import { StoreLinks } from "#lib/present/StoreLinks.ts";
import { FirstRunAdoptTextSet } from "#lib/text/FirstRunAdoptTextSet.ts";
import { FirstRunChecksTextSet } from "#lib/text/FirstRunChecksTextSet.ts";
import { I18n } from "#lib/i18n/index.ts";
import type { AdoptedInstance, ChoiceItem, SystemCheck } from "@amml/launcher-kit";
import { JobWatcher } from "./JobWatcher.ts";
import type { Navigate } from "./Navigate.ts";

export type SetupApi = Pick<
    typeof LauncherApi,
    "checksRun" | "gameInputInstall" | "openUrl" | "adoptCandidates" | "setupRun" | "instancesList" | "jobGet"
>;

// The two first-run steps: the system checks and the adoption of instances found on disk.
export class SetupStore {
    // the id of the "start from scratch" choice
    static readonly SCRATCH = "scratch";

    checks = $state<CheckView[]>([]);
    candidates = $state<AdoptCandidatesView>({ instances: [], sources: [] });
    selected = $state<string>(SetupStore.SCRATCH);
    busy = $state(false);
    fault = $state<FaultView | null>(null);

    private readonly api: SetupApi;
    private readonly events: Pick<typeof JobEvents, "subscribe">;
    private readonly navigate: Navigate;
    private readonly clock: () => number;

    readonly canContinue = $derived(CheckPresenter.canContinue(this.checks));
    readonly checksText = $derived(FirstRunChecksTextSet.build());
    readonly adoptText = $derived(FirstRunAdoptTextSet.build());
    readonly systemChecks = $derived<SystemCheck[]>(this.checks.map((check) => CheckPresenter.toSystemCheck(check)));
    readonly adopted = $derived<AdoptedInstance[]>(DataPresenter.toAdopted(this.candidates.instances));
    // the sources of worlds, then a clean start
    get choices(): ChoiceItem[] {
        const sources = DataPresenter.toSources(this.candidates.sources, { appdata: I18n.t("Store Minecraft") }, this.clock());
        return DataPresenter.toChoices(sources, {
            id: SetupStore.SCRATCH,
            title: I18n.t("Start from scratch"),
            detail: I18n.t("AMML will create a clean Minecraft Bedrock instance"),
        });
    }

    constructor(
        api: SetupApi,
        events: Pick<typeof JobEvents, "subscribe">,
        navigate: Navigate,
        clock: () => number = Date.now,
    ) {
        this.api = api;
        this.events = events;
        this.navigate = navigate;
        this.clock = clock;
    }

    async loadChecks(): Promise<void> {
        await this.guard(async () => {
            this.checks = await this.api.checksRun();
        });
    }

    // The action button of a check: install GameInput, or open the Store page of the missing app.
    async act(id: string): Promise<void> {
        const check = this.checks.find((held) => held.id === id);
        if (!check?.action) return;
        if (check.action === "install") {
            await this.guard(async () => {
                const job = await this.api.gameInputInstall();
                const ended = await JobWatcher.untilEnded(job, this.api, this.events);
                // a refused UAC prompt or an installer that did not run ends the job as failed
                if (ended.state === "failed" && ended.fault) this.fault = ended.fault;
                await this.loadChecks();
            });
            return;
        }
        await this.guard(() => this.api.openUrl(SetupStore.storeLink(check.id)));
    }

    // Next goes on only when the Store game and Gaming Services are there.
    next(): void {
        if (this.canContinue) this.navigate("/setup/adopt");
    }

    // Skip goes on whatever the checks say; the Store check runs again before an install.
    skip(): void {
        this.navigate("/setup/adopt");
    }

    async loadAdopt(): Promise<void> {
        await this.guard(async () => {
            this.candidates = await this.api.adoptCandidates();
            const first = this.candidates.sources[0];
            this.selected = first ? first.id : SetupStore.SCRATCH;
        });
    }

    select(id: string): void {
        this.selected = id;
    }

    // Copies the chosen worlds unless the choice is "scratch", then saves the settings. Nothing is adopted: this PC has no
    // AMML yet. A failed job shows its fault and leaves the step as it was.
    // With no instance afterwards the player makes one.
    async adopt(): Promise<void> {
        if (this.busy) return;
        this.busy = true;
        this.fault = null;
        await this.guard(async () => {
            const job = await this.api.setupRun(this.selected);
            const ended = await JobWatcher.untilEnded(job, this.api, this.events);
            if (ended.state !== "done") {
                // a job that did not finish never leaves a silent screen
                this.fault = ended.fault ?? { kind: "operation-failed", code: null, values: {}, restored: false };
                return;
            }
            const instances = await this.api.instancesList();
            this.navigate(instances.length === 0 ? "/instances/new" : "/instances");
        });
        this.busy = false;
    }

    back(): void {
        this.navigate("/setup/checks");
    }

    private static storeLink(id: CheckView["id"]): string {
        switch (id) {
            case "gaming-services":
                return StoreLinks.gamingServices;
            case "preview":
                return StoreLinks.preview;
            default:
                return StoreLinks.minecraft;
        }
    }

    private async guard(work: () => Promise<unknown>): Promise<void> {
        try {
            await work();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }
}
