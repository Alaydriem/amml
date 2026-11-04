import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { NameProblemView } from "#lib/bindings/NameProblemView.ts";
import type { VersionView } from "#lib/bindings/VersionView.ts";
import { VersionPresenter } from "#lib/present/VersionPresenter.ts";
import { NameProblemTextSet } from "#lib/text/NameProblemTextSet.ts";
import { NewInstanceTextSet } from "#lib/text/NewInstanceTextSet.ts";
import type { Channel, DataMode, GameVersion, NewInstanceSubmit, NewInstanceSummary } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type NewInstanceApi = Pick<
    typeof LauncherApi,
    "versionsList" | "settingsGet" | "instancesList" | "nameProblem" | "instanceCreate"
>;

// The new instance screen: the versions of a channel, the name and where the worlds live.
export class NewInstanceStore {
    // the size on disk the summary shows for a version that is not in the store yet; the real size is known once it is
    static readonly TYPICAL_BYTES = 2_000_000_000;

    channel = $state<Channel>("release");
    previewOn = $state(false);
    versions = $state<VersionView[]>([]);
    selectedVersion = $state<string | undefined>(undefined);
    name = $state("");
    dataMode = $state<DataMode>("shared");
    takenNames = $state<string[]>([]);
    problem = $state<NameProblemView | null>(null);
    fault = $state<FaultView | null>(null);

    readonly text = $derived(NewInstanceTextSet.build());
    readonly gameVersions = $derived<GameVersion[]>(this.versions.map((view) => VersionPresenter.toGameVersion(view)));
    // the kit form refuses a name that is taken; a name the bridge says is taken is passed on as taken
    readonly kitTakenNames = $derived(this.problem === "taken" ? [...this.takenNames, this.name.trim()] : this.takenNames);
    // why a bad name is refused, for the line under the field
    readonly problemText = $derived(NameProblemTextSet.text(this.problem));
    readonly chosen = $derived(this.versions.find((view) => view.version === this.selectedVersion));
    readonly summary = $derived<NewInstanceSummary>({
        ammlVersion: this.chosen?.amml ?? "",
        defaultMods: this.chosen?.defaultMods ?? 0,
        diskBytes: this.chosen?.size ?? NewInstanceStore.TYPICAL_BYTES,
    });
    readonly canCreate = $derived(
        this.problem === null &&
            this.name.trim() !== "" &&
            this.chosen !== undefined &&
            this.chosen.state !== "unsupported",
    );

    private readonly api: NewInstanceApi;
    private readonly navigate: Navigate;

    constructor(api: NewInstanceApi, navigate: Navigate) {
        this.api = api;
        this.navigate = navigate;
    }

    async load(): Promise<void> {
        try {
            const [settings, instances] = await Promise.all([this.api.settingsGet(), this.api.instancesList()]);
            this.previewOn = settings.preview;
            this.takenNames = instances.map((instance) => instance.name);
            await this.loadVersions();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    // The preview channel shows only when the settings turn preview builds on.
    channels(): Channel[] {
        return this.previewOn ? ["release", "preview"] : ["release"];
    }

    async setChannel(channel: Channel): Promise<void> {
        if (channel === "preview" && !this.previewOn) return;
        this.channel = channel;
        await this.loadVersions();
    }

    setVersion(version: string): void {
        this.selectedVersion = version;
    }

    setDataMode(mode: DataMode): void {
        this.dataMode = mode;
    }

    // Each change of the name asks the bridge whether the name can be used.
    async setName(name: string): Promise<void> {
        this.name = name;
        try {
            const problem = await this.api.nameProblem(name);
            if (this.name === name) this.problem = problem;
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    // Starts the install and shows its progress. Nothing starts while the name has a problem.
    async create(submit: NewInstanceSubmit): Promise<void> {
        if (!this.canCreate) return;
        try {
            await this.api.instanceCreate(submit.name, submit.version, this.channel === "preview", submit.dataMode === "own");
            this.navigate("/instances/install");
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    cancel(): void {
        this.navigate("/instances");
    }

    private async loadVersions(): Promise<void> {
        this.versions = await this.api.versionsList(this.channel);
        const usable = this.versions.filter((view) => view.state !== "unsupported");
        if (!usable.some((view) => view.version === this.selectedVersion)) {
            this.selectedVersion = usable[0]?.version;
        }
    }
}
