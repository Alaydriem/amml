import { ApiFault } from "#lib/api/ApiFault.ts";
import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { CardView } from "#lib/bindings/CardView.ts";
import type { CatalogLayoutView } from "#lib/bindings/CatalogLayoutView.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import type { RegistryView } from "#lib/bindings/RegistryView.ts";
import type { SearchGroupView } from "#lib/bindings/SearchGroupView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { CatalogPresenter } from "#lib/present/CatalogPresenter.ts";
import { RegistryPresenter } from "#lib/present/RegistryPresenter.ts";
import { AddModBrowseTextSet } from "#lib/text/AddModBrowseTextSet.ts";
import { UnitsTextSet } from "#lib/text/UnitsTextSet.ts";
import {
    SourceNavRules,
    Units,
    type AddModContext,
    type ModCard,
    type ModDetail,
    type ModDetailPending,
    type PageRef,
    type RegistrySource,
    type ResultGroup,
} from "@amml/launcher-kit";
import type { Navigate } from "./Navigate.ts";

export type CatalogApi = Pick<
    typeof LauncherApi,
    | "registriesRefresh"
    | "instanceGet"
    | "catalogLayout"
    | "catalogPage"
    | "catalogSearch"
    | "catalogSorted"
    | "catalogDetail"
    | "modsInstall"
>;

// The add-a-mod screen: one registry browsed by letter and page, or every registry searched.
export class CatalogStore {
    // the most pages of one registry that stay in memory
    static readonly MAX_PAGES = 10;
    // the cards a search group shows before "show more"
    static readonly SEARCH_PREVIEW = 5;

    registryViews = $state<RegistryView[]>([]);
    instance = $state<InstanceView | null>(null);
    source = $state<string>("");
    query = $state("");
    sort = $state("name");
    // counts each new source, query or sort, so the list drops the rows of the old one
    generation = $state(0);
    layout = $state<CatalogLayoutView | null>(null);
    // loaded pages: registry id, then page number from 1
    pages = $state<Record<string, Record<number, CardView[]>>>({});
    // pages asked for and not here yet
    loading = $state<PageRef[]>([]);
    searchGroups = $state<SearchGroupView[]>([]);
    expanded = $state<string[]>([]);
    sorted = $state<CardView[] | null>(null);
    checked = $state<string[]>([]);
    selected = $state<ModCard[]>([]);
    active = $state<string | undefined>(undefined);
    detail = $state<ModDetail | undefined>(undefined);
    pending = $state<ModDetailPending | undefined>(undefined);
    fault = $state<FaultView | null>(null);

    readonly text = $derived(AddModBrowseTextSet.build());
    readonly sources = $derived<RegistrySource[]>(
        this.registryViews.map((view) => RegistryPresenter.toSource(view, Date.now())),
    );
    readonly allCount = $derived(this.registryViews.reduce((sum, view) => sum + (view.modCount ?? 0), 0));
    readonly searching = $derived(this.query.trim() !== "");
    readonly context = $derived<AddModContext>({
        game: this.instance?.game ?? "",
        amml: this.instance?.ammlVersion ?? "",
        api: String(this.layout?.api ?? ""),
    });

    private readonly name: string;
    private readonly api: CatalogApi;
    private readonly navigate: Navigate;
    private readonly inFlight = new Set<string>();

    constructor(name: string, api: CatalogApi, navigate: Navigate) {
        this.name = name;
        this.api = api;
        this.navigate = navigate;
    }

    // Reads the registries and the instance, and opens the first registry.
    async load(): Promise<void> {
        try {
            this.registryViews = await this.api.registriesRefresh();
            this.instance = await this.api.instanceGet(this.name);
            const first = this.registryViews[0];
            if (first && this.source === "") await this.chooseSource(first.id);
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }

    // the rows to draw, in the kit's model
    groups(): ResultGroup[] {
        if (this.searching) return this.searchResult();
        if (this.sorted !== null && this.layout !== null) {
            const cards = this.sorted.map((card) => CatalogPresenter.toCard(card));
            return [
                {
                    id: "sorted",
                    label: "",
                    registryId: this.layout.registryId,
                    rows: cards,
                    offset: 0,
                    start: 0,
                    total: cards.length,
                    pageSize: Math.max(cards.length, 1),
                },
            ];
        }
        if (this.layout === null) return [];
        const loaded = new Map<number, ModCard[]>();
        for (const [page, cards] of Object.entries(this.pages[this.layout.registryId] ?? {})) {
            loaded.set(Number(page), cards.map((card) => CatalogPresenter.toCard(card)));
        }
        return CatalogPresenter.toGroups(this.layout, loaded);
    }

    // the line under the search field
    resultLine(): string {
        if (this.searching) {
            const total = this.searchGroups.reduce((sum, group) => sum + group.total, 0);
            return I18n.tf("{count} results in {registries} registries", {
                count: new Units(UnitsTextSet.build()).count(total),
                registries: this.searchGroups.length,
            });
        }
        const count = this.layout?.total ?? 0;
        return I18n.tf("{count} work with api {api}", {
            count: new Units(UnitsTextSet.build()).count(count),
            api: this.layout?.api ?? "",
        });
    }

    // mods in the registries other than the chosen one that match the query
    otherMatches(): number {
        return this.searchGroups
            .filter((group) => group.registryId !== this.source)
            .reduce((sum, group) => sum + group.total, 0);
    }

    async chooseSource(id: string): Promise<void> {
        if (id === SourceNavRules.URL) {
            this.navigate(`/instances/${encodeURIComponent(this.name)}/add-url`);
            return;
        }
        this.source = id;
        this.restart();
        if (id !== SourceNavRules.ALL) await this.openRegistry(id);
        if (this.searching) await this.runSearch();
    }

    async setQuery(query: string): Promise<void> {
        this.query = query;
        this.restart();
        if (this.searching) await this.runSearch();
        else this.searchGroups = [];
    }

    async setSort(id: string): Promise<void> {
        this.sort = id;
        this.restart();
        if (this.source === SourceNavRules.ALL || id === "name" || this.layout === null) {
            this.sorted = null;
            return;
        }
        await this.guard(async () => {
            const cards = await this.api.catalogSorted(this.name, this.layout?.registryId ?? "", id === "size" ? "size" : "updated");
            // a registry with too many pages keeps the name order
            this.sorted = cards;
        });
    }

    // Loads one page when it is not here and not on its way. A page that fails is forgotten, so the list asks again.
    async loadPage(ref: PageRef): Promise<void> {
        const key = `${ref.registryId}:${ref.page}`;
        if (this.pages[ref.registryId]?.[ref.page] !== undefined || this.inFlight.has(key)) return;
        this.inFlight.add(key);
        this.loading = [...this.loading, ref];
        const generation = this.generation;
        try {
            const cards = await this.api.catalogPage(this.name, ref.registryId, ref.page);
            if (generation === this.generation) this.keepPage(ref, cards);
        } catch {
            // the page leaves `loading`; the list releases it and asks for it again when it comes into view
        } finally {
            this.inFlight.delete(key);
            this.loading = this.loading.filter((held) => !(held.registryId === ref.registryId && held.page === ref.page));
        }
    }

    // the group shows all of its cards
    showMore(groupId: string): void {
        if (!this.expanded.includes(groupId)) this.expanded = [...this.expanded, groupId];
    }

    // The mod gets the detail pane at once, from its card; the document follows.
    async activate(id: string): Promise<void> {
        this.active = id;
        this.detail = undefined;
        const card = this.findCard(id);
        if (!card) return;
        const registry = this.sources.find((source) => source.id === card.registryId);
        this.pending = {
            card,
            registryName: registry?.name ?? "",
            api: this.layout?.api ?? 0,
            document: `mods/${id}.json`,
        };
        try {
            const view = await this.api.catalogDetail(this.name, card.registryId, id);
            if (this.active === id) {
                this.detail = CatalogPresenter.toDetail(view, registry?.name ?? "");
                this.pending = undefined;
            }
        } catch (error) {
            if (this.active === id) this.pending = undefined;
            this.fault = ApiFault.of(error);
        }
    }

    // The ticked ids. A tick keeps its card across a new search.
    setChecked(ids: string[]): void {
        const kept = this.selected.filter((card) => ids.includes(card.id));
        for (const id of ids) {
            if (kept.some((card) => card.id === id)) continue;
            const card = this.findCard(id);
            if (card) kept.push(card);
        }
        this.selected = kept;
        this.checked = kept.map((card) => card.id);
    }

    clearSelection(): void {
        this.selected = [];
        this.checked = [];
    }

    async install(): Promise<void> {
        if (this.checked.length === 0) return;
        await this.guard(async () => {
            await this.api.modsInstall(this.name, this.checked);
            this.navigate("/instances/install");
        });
    }

    async retrySource(id: string): Promise<void> {
        await this.guard(async () => {
            this.registryViews = await this.api.registriesRefresh();
            await this.chooseSource(id);
        });
    }

    back(): void {
        this.navigate(`/instances/${encodeURIComponent(this.name)}`);
    }

    openSettings(): void {
        this.navigate("/settings/registries");
    }

    async searchEverywhere(): Promise<void> {
        await this.chooseSource(SourceNavRules.ALL);
    }

    async clearSearch(): Promise<void> {
        await this.setQuery("");
    }

    private restart(): void {
        this.generation += 1;
        this.sorted = null;
        this.expanded = [];
    }

    private async openRegistry(id: string): Promise<void> {
        const generation = this.generation;
        try {
            const layout = await this.api.catalogLayout(this.name, id);
            // a layout that comes after the player chose another source is dropped
            if (generation !== this.generation) return;
            this.layout = layout;
            this.pages = { ...this.pages, [id]: {} };
            if (layout.total > 0) await this.loadPage({ registryId: id, page: 1 });
        } catch (error) {
            if (generation !== this.generation) return;
            // a registry that is offline has its own view; any other failure is a fault
            const status = this.registryViews.find((view) => view.id === id)?.status;
            this.layout = null;
            if (status !== "offline" && status !== "broken") this.fault = ApiFault.of(error);
        }
    }

    private async runSearch(): Promise<void> {
        const generation = this.generation;
        await this.guard(async () => {
            const groups = await this.api.catalogSearch(this.name, this.query);
            if (generation === this.generation) this.searchGroups = groups;
        });
    }

    // Keeps the page and drops the page furthest from it when a registry holds more than `MAX_PAGES`.
    private keepPage(ref: PageRef, cards: CardView[]): void {
        const held = { ...(this.pages[ref.registryId] ?? {}), [ref.page]: cards };
        const numbers = Object.keys(held).map(Number);
        while (numbers.length > CatalogStore.MAX_PAGES) {
            let furthest = numbers[0] ?? ref.page;
            for (const number of numbers) {
                if (Math.abs(number - ref.page) > Math.abs(furthest - ref.page)) furthest = number;
            }
            delete held[furthest];
            numbers.splice(numbers.indexOf(furthest), 1);
        }
        this.pages = { ...this.pages, [ref.registryId]: held };
    }

    private searchResult(): ResultGroup[] {
        const groups = this.source === SourceNavRules.ALL ? this.searchGroups : this.searchGroups.filter((group) => group.registryId === this.source);
        return groups.map((group) => {
            const cards = group.cards.map((card) => CatalogPresenter.toCard(card));
            const all = this.expanded.includes(group.registryId);
            const rows = all ? cards : cards.slice(0, CatalogStore.SEARCH_PREVIEW);
            const registry = this.sources.find((source) => source.id === group.registryId);
            return {
                id: group.registryId,
                label: registry?.name ?? group.registryId,
                registryId: group.registryId,
                rows,
                offset: 0,
                start: 0,
                total: all ? rows.length : group.total,
                pageSize: Math.max(rows.length, 1),
                collapsedMore: all ? undefined : Math.max(0, group.total - rows.length),
            };
        });
    }

    private findCard(id: string): ModCard | undefined {
        const kept = this.selected.find((card) => card.id === id);
        if (kept) return kept;
        const fromPages = Object.values(this.pages).flatMap((byPage) => Object.values(byPage).flat());
        const found = [...fromPages, ...(this.sorted ?? []), ...this.searchGroups.flatMap((group) => group.cards)].find(
            (card) => card.id === id,
        );
        return found ? CatalogPresenter.toCard(found) : undefined;
    }

    private async guard(work: () => Promise<unknown>): Promise<void> {
        try {
            await work();
        } catch (error) {
            this.fault = ApiFault.of(error);
        }
    }
}
