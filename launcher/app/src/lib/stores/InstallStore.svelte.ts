import { ApiFault } from "#lib/api/ApiFault.ts";
import type { JobEvents } from "#lib/api/JobEvents.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";
import { FaultPresenter, type PresentedFault } from "#lib/present/FaultPresenter.ts";
import { DialogsTextSet } from "#lib/text/DialogsTextSet.ts";
import { InstallProgressTextSet } from "#lib/text/InstallProgressTextSet.ts";
import { StepTextSet } from "#lib/text/StepTextSet.ts";
import type { InstallProgress, InstallStep } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type InstallApi = Pick<
    typeof LauncherApi,
    "jobGet" | "jobCancel" | "jobRetry" | "jobRetryDeveloperMode" | "openPath"
>;

// The install progress screen. It shows the running job at once, from the job's own snapshot, and follows it.
export class InstallStore {
    // the file Windows would not let the launcher copy
    static readonly GAME_EXE = "Minecraft.Windows.exe";

    snapshot = $state<JobSnapshot | null>(null);
    // the Developer Mode question is open
    asking = $state(false);
    // a fault the screen shows in place of the job's own, after the player says no to Developer Mode
    override = $state<FaultView | null>(null);
    loaded = $state(false);
    // the player pressed Cancel and the job has not stopped yet
    cancelling = $state(false);
    failure = $state<FaultView | null>(null);

    readonly text = $derived(InstallProgressTextSet.build());
    readonly steps = $derived<InstallStep[]>(
        this.snapshot ? StepTextSet.toInstallSteps(this.snapshot.steps, this.snapshot.version) : [],
    );
    readonly progress = $derived<InstallProgress | undefined>(InstallStore.readout(this.snapshot));
    readonly fault = $derived<PresentedFault | undefined>(this.presented());
    readonly developerModeText = $derived(DialogsTextSet.build().developerMode);
    readonly developerModeValues = $derived({
        file: InstallStore.GAME_EXE,
        version: this.snapshot?.fault?.values.version ?? this.snapshot?.version ?? "",
    });

    private readonly api: InstallApi;
    private readonly events: Pick<typeof JobEvents, "subscribe">;
    private readonly navigate: Navigate;
    private readonly logs: () => Promise<void>;
    private stop: (() => void) | undefined;

    constructor(
        api: InstallApi,
        events: Pick<typeof JobEvents, "subscribe">,
        navigate: Navigate,
        logs: () => Promise<void>,
    ) {
        this.api = api;
        this.events = events;
        this.navigate = navigate;
        this.logs = logs;
    }

    // Reads the job that is running (or the last one that ended) and follows it from then on.
    async start(): Promise<void> {
        this.stop = await this.events.subscribe((next) => this.apply(next));
        try {
            const current = await this.api.jobGet();
            if (current && (this.snapshot === null || current.id >= this.snapshot.id)) this.apply(current);
        } catch (error) {
            this.failure = ApiFault.of(error);
        }
        this.loaded = true;
    }

    stopWatching(): void {
        this.stop?.();
        this.stop = undefined;
    }

    async cancel(): Promise<void> {
        this.cancelling = true;
        try {
            await this.api.jobCancel();
        } catch (error) {
            // the cancel did not reach the job, so the screen is not "cancelling"
            this.cancelling = false;
            this.failure = ApiFault.of(error);
        }
    }

    async retry(): Promise<void> {
        this.override = null;
        await this.guard(async () => {
            await this.api.jobRetry();
            await this.refresh();
        });
    }

    async showLog(): Promise<void> {
        await this.logs();
    }

    back(): void {
        this.navigate("/instances");
    }

    // yes to the Developer Mode question: turn it on and run the install once more
    async allowDeveloperMode(): Promise<void> {
        this.asking = false;
        this.override = null;
        await this.guard(async () => {
            await this.api.jobRetryDeveloperMode();
            await this.refresh();
        });
    }

    // no to the question: the screen shows the copy failure
    refuseDeveloperMode(): void {
        this.asking = false;
        const original = this.snapshot?.fault;
        this.override = { kind: "copy-failed", code: null, values: original?.values ?? {}, restored: original?.restored ?? true };
    }

    // follows a snapshot; a newer job replaces an older one
    apply(next: JobSnapshot): void {
        if (this.snapshot !== null && next.id < this.snapshot.id) return;
        if (next.state !== "running" || (this.snapshot !== null && next.id > this.snapshot.id)) this.cancelling = false;
        this.snapshot = next;
        if (next.state === "cancelled") {
            this.navigate("/instances");
        } else if (next.state === "done") {
            this.navigate(InstallStore.returnTo(next));
        } else if (next.state === "failed" && next.fault?.kind === "consent-needed" && this.override === null) {
            this.asking = true;
        }
    }

    // where the player goes when a job is done
    static returnTo(snapshot: JobSnapshot): string {
        switch (snapshot.kind) {
            case "create-instance":
            case "update-amml":
            case "install-mods":
            case "install-url":
            case "update-mods":
                return `/instances/${encodeURIComponent(snapshot.titleName)}`;
            case "copy-worlds":
                return "/settings/shared";
            default:
                return "/instances";
        }
    }

    private presented(): PresentedFault | undefined {
        const view = this.override ?? this.snapshot?.fault ?? null;
        if (view === null || view.kind === "consent-needed") return undefined;
        return FaultPresenter.present(view);
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

    private async refresh(): Promise<void> {
        const current = await this.api.jobGet();
        if (current) this.snapshot = current;
    }

    private async guard(work: () => Promise<unknown>): Promise<void> {
        try {
            await work();
        } catch (error) {
            this.failure = ApiFault.of(error);
        }
    }
}
