import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import type { UrlCheckView } from "#lib/bindings/UrlCheckView.ts";
import { RegistryPresenter } from "#lib/present/RegistryPresenter.ts";
import { AddModUrlTextSet } from "#lib/text/AddModUrlTextSet.ts";
import { UrlCheckTextSet } from "#lib/text/UrlCheckTextSet.ts";
import { SourceNavRules, type AddModContext, type FoundMod, type RegistrySource, type UrlCheck } from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type UrlInstallApi = Pick<typeof LauncherApi, "registriesRefresh" | "instanceGet" | "urlInspect" | "urlInstall">;

// The add-a-mod-from-a-link screen: look at the zip, then install it.
export class UrlInstallStore {
    url = $state("");
    checking = $state(false);
    check = $state<UrlCheckView | null>(null);
    registryViews = $state<RegistryView[]>([]);
    instance = $state<InstanceView | null>(null);
    fault = $state<FaultView | null>(null);

    readonly text = $derived(AddModUrlTextSet.build());
    readonly sources = $derived<RegistrySource[]>(
        this.registryViews.map((view) => RegistryPresenter.toSource(view, Date.now())),
    );
    readonly allCount = $derived(this.registryViews.reduce((sum, view) => sum + (view.modCount ?? 0), 0));
    readonly checks = $derived<UrlCheck[]>(UrlCheckTextSet.rows(this.url, this.check));
    readonly found = $derived<FoundMod | undefined>(UrlCheckTextSet.found(this.check));
    // a zip no registry lists installs as unofficial and needs the player's consent
    readonly unofficial = $derived(this.check !== null && this.check.matchedRegistry === null);
    readonly context = $derived<AddModContext>({
        game: this.instance?.game ?? "",
        amml: this.instance?.ammlVersion ?? "",
        api: String(this.check?.api ?? ""),
    });

    private readonly name: string;
    private readonly api: UrlInstallApi;
    private readonly navigate: Navigate;

    constructor(name: string, api: UrlInstallApi, navigate: Navigate) {
        this.name = name;
        this.api = api;
        this.navigate = navigate;
    }

    async load(): Promise<void> {
        try {
            this.registryViews = await this.api.registriesRefresh();
            this.instance = await this.api.instanceGet(this.name);
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    // typing a new link drops the checks of the old one
    setUrl(url: string): void {
        this.url = url;
        this.check = null;
    }

    async inspect(url: string): Promise<void> {
        this.url = url;
        this.checking = true;
        this.fault = null;
        try {
            this.check = await this.api.urlInspect(this.name, url.trim());
        } catch (error) {
            this.check = null;
            this.fault = ApiFault.of(error);
        } finally {
            this.checking = false;
        }
    }

    // Installs the zip. A zip no registry lists goes in only with the player's consent, which pressing the warn
    // button of the bar is.
    async install(): Promise<void> {
        if (this.check === null || !this.check.apiMatches) return;
        try {
            await this.api.urlInstall(this.name, this.url.trim(), this.unofficial);
            this.navigate("/instances/install");
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    chooseSource(id: string): void {
        if (id === SourceNavRules.URL) return;
        this.navigate(`/instances/${encodeURIComponent(this.name)}/add`);
    }

    back(): void {
        this.navigate(`/instances/${encodeURIComponent(this.name)}/add`);
    }
}
