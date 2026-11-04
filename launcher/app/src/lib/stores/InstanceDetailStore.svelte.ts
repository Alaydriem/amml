import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { ModView } from "#lib/bindings/ModView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import { InstancePresenter } from "#lib/present/InstancePresenter.ts";
import { ModPresenter } from "#lib/present/ModPresenter.ts";
import { DialogsTextSet } from "#lib/text/DialogsTextSet.ts";
import { FirstRunAdoptTextSet } from "#lib/text/FirstRunAdoptTextSet.ts";
import { InstanceDetailTextSet } from "#lib/text/InstanceDetailTextSet.ts";
import { InstancesTextSet } from "#lib/text/InstancesTextSet.ts";
import { ModNoteTextSet } from "#lib/text/ModNoteTextSet.ts";
import type { InstalledMod, Instance } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type InstanceDetailApi = Pick<
    typeof LauncherApi,
    | "instanceGet"
    | "modsList"
    | "modSetEnabled"
    | "modRemove"
    | "modsUpdate"
    | "ammlUpdate"
    | "openPath"
    | "instanceLaunch"
    | "instanceDelete"
>;

// The page of one instance: its mods and the actions on them.
export class InstanceDetailStore {
    instance = $state<InstanceView | null>(null);
    mods = $state<ModView[]>([]);
    loaded = $state(false);
    fault = $state<FaultView | null>(null);
    // the mod the player asked to remove and has not confirmed yet
    pendingRemove = $state<ModView | null>(null);
    pendingDelete = $state(false);

    readonly text = $derived(InstanceDetailTextSet.build());
    readonly headerText = $derived(InstancesTextSet.instanceHeader());
    readonly removeText = $derived(DialogsTextSet.removeMod());
    readonly deleteText = $derived(DialogsTextSet.build().deleteInstance);

    private readonly name: string;
    private readonly api: InstanceDetailApi;
    private readonly navigate: Navigate;
    private readonly registries: () => readonly RegistryView[];
    private readonly clock: () => number;

    constructor(
        name: string,
        api: InstanceDetailApi,
        navigate: Navigate,
        registries: () => readonly RegistryView[],
        clock: () => number = Date.now,
    ) {
        this.name = name;
        this.api = api;
        this.navigate = navigate;
        this.registries = registries;
        this.clock = clock;
    }

    async load(): Promise<void> {
        this.fault = null;
        try {
            this.instance = await this.api.instanceGet(this.name);
            this.mods = await this.api.modsList(this.name);
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
        this.loaded = true;
    }

    // the mods in the kit's model
    installed(): InstalledMod[] {
        const notes = ModNoteTextSet.build();
        return this.mods.map((view) => ModPresenter.toInstalled(view, this.registries(), notes));
    }

    // the instance in the kit's model, for the header
    header(): Instance | null {
        if (this.instance === null) return null;
        const unknown = { unknownVersion: FirstRunAdoptTextSet.build().adopted.unknownVersion };
        return InstancePresenter.toInstance(this.instance, unknown, this.clock());
    }

    async toggle(id: string, enabled: boolean): Promise<void> {
        await this.guard(() => this.api.modSetEnabled(this.name, id, enabled));
        await this.load();
    }

    // The AMML row updates AMML; any other row updates its mod. The install screen shows the job.
    async update(id: string): Promise<void> {
        await this.guard(async () => {
            if (id === ModPresenter.AMML_ID) await this.api.ammlUpdate(this.name);
            else await this.api.modsUpdate(this.name, [id]);
            this.navigate("/instances/install");
        });
    }

    // every mod with an update, then AMML last
    updateAllIds(): string[] {
        const waiting = this.mods.filter((mod) => mod.update === "available");
        const mods = waiting.filter((mod) => mod.id !== ModPresenter.AMML_ID).map((mod) => mod.id);
        const amml = waiting.some((mod) => mod.id === ModPresenter.AMML_ID) ? [ModPresenter.AMML_ID] : [];
        return [...mods, ...amml];
    }

    async updateAll(): Promise<void> {
        const ids = this.updateAllIds();
        if (ids.length === 0) return;
        await this.guard(async () => {
            await this.api.modsUpdate(this.name, ids);
            this.navigate("/instances/install");
        });
    }

    add(): void {
        this.navigate(`/instances/${encodeURIComponent(this.name)}/add`);
    }

    back(): void {
        this.navigate("/instances");
    }

    async menuAction(id: string, itemId: string): Promise<void> {
        const mod = this.mods.find((held) => held.id === id);
        if (!mod || this.instance === null) return;
        if (itemId === "folder") {
            const side = mod.enabled ? "amml" : "amml-disabled";
            await this.guard(() => this.api.openPath(`${this.instance?.folder}\\mods\\${side}\\${id.replace("/", "\\")}`));
        } else if (itemId === "remove") {
            this.pendingRemove = mod;
        }
    }

    async confirmRemove(): Promise<void> {
        const mod = this.pendingRemove;
        if (!mod) return;
        this.pendingRemove = null;
        await this.guard(() => this.api.modRemove(this.name, mod.id));
        await this.load();
    }

    cancelRemove(): void {
        this.pendingRemove = null;
    }

    async launch(): Promise<void> {
        await this.guard(() => this.api.instanceLaunch(this.name));
        await this.load();
    }

    async openFolder(): Promise<void> {
        if (this.instance) await this.guard(() => this.api.openPath(this.instance?.folder ?? ""));
    }

    askDelete(): void {
        this.pendingDelete = true;
    }

    cancelDelete(): void {
        this.pendingDelete = false;
    }

    // the folder goes only after this call, with the instance name as the confirmation text
    async confirmDelete(): Promise<void> {
        this.pendingDelete = false;
        let deleted = false;
        await this.guard(async () => {
            await this.api.instanceDelete(this.name, this.name);
            deleted = true;
        });
        if (deleted) this.navigate("/instances");
    }

    private async guard(work: () => Promise<unknown>): Promise<void> {
        try {
            this.fault = null;
            await work();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }
}
