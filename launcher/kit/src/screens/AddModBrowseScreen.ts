import type {
    AddModContext,
    ModCard,
    ModDetail,
    ModDetailPending,
    PageRef,
    RegistrySource,
    ResultGroup,
} from "../model";
import type { AddModHeaderText } from "../components/catalog/AddModHeader";
import type { CatalogToolbarText } from "../components/catalog/CatalogToolbar";
import type { ModDetailPaneText } from "../components/catalog/ModDetailPane";
import type { ModResultListText } from "../components/catalog/ModResultList";
import type { SelectionBarText } from "../components/catalog/SelectionBar";
import type { SentinelObserverFactory } from "../components/catalog/SentinelObserverFactory";
import type { SourceNavText } from "../components/catalog/SourceNav";
import { SourceNavRules } from "../components/catalog/SourceNav";
import { Template, type Plural, type TextPart } from "../text/Template";

export interface AddModBrowseScreenText {
    header: AddModHeaderText;
    toolbar: CatalogToolbarText;
    /** Template with a `{name}` slot: the search placeholder when one registry is chosen. */
    searchIn: string;
    /** The search placeholder when every registry is searched. */
    searchAll: string;
    nav: SourceNavText;
    list: ModResultListText;
    detail: ModDetailPaneText;
    bar: SelectionBarText;
    /** The state of a registry the launcher could not reach (artboard 06e). */
    offline: {
        /** Template with a `{name}` slot: "{name} is offline." */
        title: string;
        /** Template with a `{url}` slot; `**…**` marks the address. */
        body: string;
        retry: string;
        settings: string;
        /** Template with a `{fetched}` slot: the line under the search field, "No list · last tried {fetched}". */
        line: string;
        /** The line when the registry was never reached. */
        lineNever: string;
    };
    /** The state of a search with no match (artboard 06f). */
    noResults: {
        /** Template with `{name}` and `{query}` slots: one registry is chosen. */
        title: string;
        /** Template with a `{query}` slot: every registry is searched. */
        titleAll: string;
        /** Plural with a `{count}` slot: what the other registries hold. */
        body: Plural;
        /** The body when no other registry has a match. */
        bodyNone: string;
        searchAll: string;
        clear: string;
    };
}

/** Which of the three bodies the middle of the screen shows. */
export type AddModBrowseView = "list" | "offline" | "empty";

export interface AddModBrowseScreenProps {
    /** Name of the instance the mods go to, shown strong in the title. */
    name: string;
    context: AddModContext;
    registries: RegistrySource[];
    allCount?: number;
    /** Chosen source: a registry id or `SourceNavRules.ALL`. Search mode is all registries with a query. */
    source?: string;
    showOtherAmml?: boolean;
    /** Shown search text. The screen never changes it: the app answers `onsearch` with a new value. */
    query?: string;
    sort?: string;
    /** The mono line under the search field. The app builds it. An offline registry shows its own line. */
    resultLine: string;
    /** Rows to show. The app filters and sorts; the screen does not. */
    groups: ResultGroup[];
    /** Ids of the ticked mods. */
    checked?: string[];
    /** The ticked mods as cards. The app supplies them, so a tick survives a new search. */
    selectedCards: ModCard[];
    /** Id of the mod in the detail pane. */
    active?: string;
    detail?: ModDetail;
    /** The mod whose document is loading; the pane shows its loading state. */
    pending?: ModDetailPending;
    loading?: PageRef[];
    /** Counts the generation of the shown rows; see `ModResultListProps.generation`. */
    generation?: number;
    /** Mods in the other registries that match the query; the no-results state names the count. */
    otherMatches?: number;
    /** Search mode: group heads show the row totals. */
    showGroupCount?: boolean;
    text: AddModBrowseScreenText;
    /** Replaces the `IntersectionObserver` wrapper of the list. */
    observerFactory?: SentinelObserverFactory;
    onback?: () => void;
    onsource?: (id: string) => void;
    onshowotheramml?: (show: boolean) => void;
    onsearch?: (query: string) => void;
    onsort?: (id: string) => void;
    /** The new ticked ids. */
    onchecked?: (ids: string[]) => void;
    onloadmore?: (page: PageRef) => void;
    onshowmore?: (groupId: string) => void;
    onactivate?: (id: string) => void;
    onclear?: () => void;
    oninstall?: () => void;
    /** Try Again in the offline state, with the id of the registry. */
    onretrysource?: (id: string) => void;
    /** Registry settings in the offline state. */
    onopensettings?: () => void;
    /** Search every registry in the no-results state. */
    onsearchall?: () => void;
    /** Clear search in the no-results state. */
    onclearsearch?: () => void;
}

export class AddModBrowseScreenRules {
    /** The registry the chosen source names, or undefined for all registries. */
    static registry(source: string, registries: readonly RegistrySource[]): RegistrySource | undefined {
        return registries.find((registry) => registry.id === source);
    }

    static view(
        source: string,
        registries: readonly RegistrySource[],
        groups: readonly ResultGroup[],
        query: string,
    ): AddModBrowseView {
        if (AddModBrowseScreenRules.registry(source, registries)?.status === "offline") return "offline";
        if (query.trim() !== "" && groups.every((group) => group.total === 0)) return "empty";
        return "list";
    }

    static placeholder(source: string, registries: readonly RegistrySource[], text: AddModBrowseScreenText): string {
        const registry = AddModBrowseScreenRules.registry(source, registries);
        return registry ? Template.fill(text.searchIn, { name: registry.name }) : text.searchAll;
    }

    /** The line under the search field: the app's line, or the offline line of the chosen registry. */
    static line(
        view: AddModBrowseView,
        registry: RegistrySource | undefined,
        resultLine: string,
        text: AddModBrowseScreenText,
    ): string {
        if (view !== "offline" || !registry) return resultLine;
        return registry.fetched === undefined
            ? text.offline.lineNever
            : Template.fill(text.offline.line, { fetched: registry.fetched });
    }

    static offlineTitle(registry: RegistrySource, text: AddModBrowseScreenText): string {
        return Template.fill(text.offline.title, { name: registry.name });
    }

    static offlineBody(registry: RegistrySource, text: AddModBrowseScreenText): TextPart[] {
        return Template.parts(text.offline.body, { url: registry.url });
    }

    static noResultsTitle(registry: RegistrySource | undefined, query: string, text: AddModBrowseScreenText): string {
        const term = query.trim();
        return registry
            ? Template.fill(text.noResults.title, { name: registry.name, query: term })
            : Template.fill(text.noResults.titleAll, { query: term });
    }

    static noResultsBody(otherMatches: number, text: AddModBrowseScreenText): string {
        if (otherMatches <= 0) return text.noResults.bodyNone;
        return Template.plural(text.noResults.body, otherMatches);
    }

    /** Search every registry only helps when one registry is chosen and others have matches. */
    static canSearchAll(source: string, otherMatches: number): boolean {
        return source !== SourceNavRules.ALL && otherMatches > 0;
    }
}
