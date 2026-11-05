import { ApiFault } from "#lib/api/ApiFault.ts";
import type { JobEvents } from "#lib/api/JobEvents.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";
import { I18n } from "#lib/i18n/index.ts";
import { FaultPresenter, type PresentedFault } from "#lib/present/FaultPresenter.ts";
import { SetupProgressTextSet } from "#lib/text/SetupProgressTextSet.ts";
import { StepTextSet } from "#lib/text/StepTextSet.ts";
import type { InstallProgress, InstallStep } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type SetupProgressApi = Pick<typeof LauncherApi, "jobGet" | "jobRetry" | "instancesList" | "openPath">;

// The first-run copy progress screen. It follows the setup job and goes on when the job is done.
export class SetupProgressStore {
    // the job kind this screen follows
    static readonly KIND = "setup-data";
    // the source id of the Store Minecraft data folder
    static readonly APPDATA = "appdata";

    snapshot = $state<JobSnapshot | null>(null);
    loaded = $state(false);
    failure = $state<FaultView | null>(null);

    readonly text = $derived(SetupProgressTextSet.build(this.snapshot?.fault?.restored ?? true));
    readonly steps = $derived<InstallStep[]>(
        this.snapshot ? StepTextSet.toInstallSteps(this.snapshot.steps, this.snapshot.version) : [],
    );
    readonly progress = $derived<InstallProgress | undefined>(SetupProgressStore.readout(this.snapshot));
    readonly fault = $derived<PresentedFault | undefined>(
        this.snapshot?.fault ? FaultPresenter.present(this.snapshot.fault) : undefined,
    );
    readonly sourceName = $derived(
        this.snapshot?.titleName === SetupProgressStore.APPDATA ? I18n.t("Store Minecraft") : (this.snapshot?.titleName ?? ""),
    );

    private readonly api: SetupProgressApi;
    private readonly events: Pick<typeof JobEvents, "subscribe">;
    private readonly navigate: Navigate;
    private readonly logs: () => Promise<void>;
    private stop: (() => void) | undefined;

    constructor(
        api: SetupProgressApi,
        events: Pick<typeof JobEvents, "subscribe">,
        navigate: Navigate,
        logs: () => Promise<void>,
    ) {
        this.api = api;
        this.events = events;
        this.navigate = navigate;
        this.logs = logs;
    }

    // Reads the setup job and follows it. With no setup job the player is sent back to the choice.
    async start(): Promise<void> {
        this.stop = await this.events.subscribe((next) => void this.apply(next));
        try {
            const current = await this.api.jobGet();
            if (current !== null && current.kind !== SetupProgressStore.KIND) {
                this.navigate("/setup/adopt");
            } else if (current !== null) {
                if (this.snapshot === null || current.id >= this.snapshot.id) await this.apply(current);
            } else if (this.snapshot === null) {
                this.navigate("/setup/adopt");
            }
        } catch (error) {
            this.failure = ApiFault.of(error);
        }
        this.loaded = true;
    }

    stopWatching(): void {
        this.stop?.();
        this.stop = undefined;
    }

    async retry(): Promise<void> {
        try {
            await this.api.jobRetry();
            const current = await this.api.jobGet();
            if (current) this.snapshot = current;
        } catch (error) {
            this.failure = ApiFault.of(error);
        }
    }

    chooseAnother(): void {
        this.navigate("/setup/adopt");
    }

    async showLog(): Promise<void> {
        await this.logs();
    }

    // follows a snapshot; a newer job replaces an older one
    async apply(next: JobSnapshot): Promise<void> {
        if (next.kind !== SetupProgressStore.KIND) return;
        if (this.snapshot !== null && next.id < this.snapshot.id) return;
        this.snapshot = next;
        if (next.state !== "done") return;
        try {
            const instances = await this.api.instancesList();
            this.navigate(instances.length === 0 ? "/instances/new" : "/instances");
        } catch (error) {
            this.failure = ApiFault.of(error);
        }
    }

    private static readout(snapshot: JobSnapshot | null): InstallProgress | undefined {
        const progress = snapshot?.progress;
        if (!progress) return undefined;
        return {
            done: progress.done,
            total: progress.total,
            rate: progress.rate ?? undefined,
            remaining: progress.remainingSeconds ?? undefined,
            detail: progress.file ?? undefined,
        };
    }
}
