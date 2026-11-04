import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import { InstancePresenter } from "#lib/present/InstancePresenter.ts";
import { DialogsTextSet } from "#lib/text/DialogsTextSet.ts";
import { FirstRunAdoptTextSet } from "#lib/text/FirstRunAdoptTextSet.ts";
import { InstancesTextSet } from "#lib/text/InstancesTextSet.ts";
import type { Instance } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type InstancesApi = Pick<typeof LauncherApi, "instancesList" | "instanceLaunch" | "instanceDelete" | "openPath">;

// The instance list: launch, open, open the folder and delete.
export class InstancesStore {
    // how often the list is read while a game runs
    static readonly POLL_MILLIS = 2000;

    views = $state<InstanceView[]>([]);
    loaded = $state(false);
    // the instance the player asked to delete and has not confirmed yet
    pendingDelete = $state<InstanceView | null>(null);
    fault = $state<FaultView | null>(null);

    readonly text = $derived(InstancesTextSet.build());
    readonly menu = $derived(InstancesTextSet.instanceMenu());
    readonly deleteText = $derived(DialogsTextSet.build().deleteInstance);

    private readonly api: InstancesApi;
    private readonly navigate: Navigate;
    private readonly clock: () => number;
    private timer: ReturnType<typeof setInterval> | undefined;

    constructor(api: InstancesApi, navigate: Navigate, clock: () => number = Date.now) {
        this.api = api;
        this.navigate = navigate;
        this.clock = clock;
    }

    // the instances in the kit's model, worded for now
    instances(): Instance[] {
        const unknown = { unknownVersion: FirstRunAdoptTextSet.build().adopted.unknownVersion };
        const now = this.clock();
        return this.views.map((view) => InstancePresenter.toInstance(view, unknown, now));
    }

    async load(): Promise<void> {
        try {
            this.views = await this.api.instancesList();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
        this.loaded = true;
        this.schedule();
    }

    async launch(name: string): Promise<void> {
        try {
            this.fault = null;
            await this.api.instanceLaunch(name);
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
        await this.load();
    }

    open(name: string): void {
        this.navigate(`/instances/${encodeURIComponent(name)}`);
    }

    newInstance(): void {
        this.navigate("/instances/new");
    }

    // the banner's action opens the first instance that has updates
    review(): void {
        const first = this.views.find((view) => view.updateCount > 0);
        if (first) this.open(first.name);
    }

    async menuAction(name: string, itemId: string): Promise<void> {
        const view = this.views.find((held) => held.name === name);
        if (!view) return;
        if (itemId === "open-folder") {
            try {
                await this.api.openPath(view.folder);
            } catch (error) {
                this.fault = ApiFault.of(error);
            }
        } else if (itemId === "delete") {
            this.pendingDelete = view;
        }
    }

    // The folder goes only after this call, with the instance name as the confirmation text.
    async confirmDelete(): Promise<void> {
        const view = this.pendingDelete;
        if (!view) return;
        this.pendingDelete = null;
        try {
            await this.api.instanceDelete(view.name, view.name);
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
        await this.load();
    }

    cancelDelete(): void {
        this.pendingDelete = null;
    }

    dispose(): void {
        this.stopPolling();
    }

    // The list is read again every two seconds while some instance runs, and not otherwise.
    private schedule(): void {
        const running = this.views.some((view) => view.runningSince !== null);
        if (running && this.timer === undefined) {
            this.timer = setInterval(() => void this.load(), InstancesStore.POLL_MILLIS);
        } else if (!running) {
            this.stopPolling();
        }
    }

    private stopPolling(): void {
        if (this.timer !== undefined) {
            clearInterval(this.timer);
            this.timer = undefined;
        }
    }
}
