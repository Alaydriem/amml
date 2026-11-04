import { fireEvent, render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import AddModBrowseScreen from "../../src/screens/AddModBrowseScreen.svelte";
import { AddModBrowseScreenRules, type AddModBrowseScreenProps } from "../../src/screens/AddModBrowseScreen";
import { CatalogFixtures } from "../../src/fixtures/catalog";
import { addModBrowseEn } from "../../src/text/en/addModBrowse";
import { FakeObserver } from "../components/catalog/FakeObserver";

const text = addModBrowseEn;
const registries = CatalogFixtures.registries;
const community = registries[1]!;
const offline = registries[3]!;

function props(overrides: Partial<AddModBrowseScreenProps> = {}): AddModBrowseScreenProps {
    return {
        name: CatalogFixtures.instanceName,
        context: CatalogFixtures.context,
        registries,
        allCount: CatalogFixtures.allCount,
        source: "community",
        resultLine: CatalogFixtures.browseLine,
        groups: CatalogFixtures.browseGroups(),
        selectedCards: [],
        text,
        observerFactory: new FakeObserver().factory,
        ...overrides,
    };
}

describe("AddModBrowseScreenRules", () => {
    const groups = CatalogFixtures.browseGroups();

    it("shows the offline state for an offline registry, whatever the query", () => {
        expect(AddModBrowseScreenRules.view("old", registries, [], "")).toBe("offline");
        expect(AddModBrowseScreenRules.view("old", registries, groups, "map")).toBe("offline");
    });

    it("shows the no-results state only for a query that matched nothing", () => {
        expect(AddModBrowseScreenRules.view("community", registries, [], "dynmap")).toBe("empty");
        expect(AddModBrowseScreenRules.view("community", registries, [], "   ")).toBe("list");
        expect(AddModBrowseScreenRules.view("community", registries, groups, "map")).toBe("list");
        expect(AddModBrowseScreenRules.view("all", registries, [], "dynmap")).toBe("empty");
    });

    it("names the chosen registry in the placeholder, or all registries", () => {
        expect(AddModBrowseScreenRules.placeholder("community", registries, text)).toBe("Search Community index");
        expect(AddModBrowseScreenRules.placeholder("all", registries, text)).toBe("Search all registries");
    });

    it("builds the offline line from the last try, or says there is no list", () => {
        expect(AddModBrowseScreenRules.line("offline", offline, "ignored", text)).toBe("No list · last tried 2 min ago");
        expect(AddModBrowseScreenRules.line("offline", { ...offline, fetched: undefined }, "ignored", text)).toBe("No list");
        expect(AddModBrowseScreenRules.line("list", community, "1,412 work", text)).toBe("1,412 work");
    });

    it("counts the other registries in the no-results body", () => {
        expect(AddModBrowseScreenRules.noResultsBody(4, text)).toContain("4 mods in your other registries match.");
        expect(AddModBrowseScreenRules.noResultsBody(1, text)).toContain("1 mod in your other registries matches.");
        expect(AddModBrowseScreenRules.noResultsBody(0, text)).not.toContain("other registries");
    });

    it("offers Search every registry only for one registry with matches elsewhere", () => {
        expect(AddModBrowseScreenRules.canSearchAll("community", 4)).toBe(true);
        expect(AddModBrowseScreenRules.canSearchAll("community", 0)).toBe(false);
        expect(AddModBrowseScreenRules.canSearchAll("all", 4)).toBe(false);
    });
});

describe("AddModBrowseScreen", () => {
    it("shows the offline state with the address, locked search and sort, and the callbacks", async () => {
        const onretrysource = vi.fn();
        const onopensettings = vi.fn();
        render(AddModBrowseScreen, { props: props({ source: "old", groups: [], onretrysource, onopensettings }) });

        expect(screen.getByRole("heading", { name: "Old test registry is offline." })).toBeInTheDocument();
        expect(screen.getByText("example.net/amml-test/registry.json").tagName).toBe("STRONG");
        expect(screen.getByText("No list · last tried 2 min ago")).toBeInTheDocument();
        expect(screen.getByRole("searchbox", { name: "Search mods" })).toBeDisabled();
        expect(screen.getByRole("combobox", { name: "Sort by" })).toBeDisabled();
        expect(screen.queryByRole("region", { name: "Mods" })).toBeNull();

        await fireEvent.click(screen.getByRole("button", { name: "Try again" }));
        await fireEvent.click(screen.getByRole("button", { name: "Registry settings" }));

        expect(onretrysource).toHaveBeenCalledExactlyOnceWith("old");
        expect(onopensettings).toHaveBeenCalledOnce();
    });

    it("shows the no-results state with the other-registry count and the callbacks", async () => {
        const onsearchall = vi.fn();
        const onclearsearch = vi.fn();
        render(AddModBrowseScreen, {
            props: props({
                groups: [],
                query: "dynmap",
                resultLine: CatalogFixtures.noResultsLine,
                otherMatches: 4,
                onsearchall,
                onclearsearch,
            }),
        });

        expect(screen.getByRole("heading", { name: "No mods in Community index match “dynmap”." })).toBeInTheDocument();
        expect(screen.getByText(/4 mods in your other registries match\./)).toBeInTheDocument();
        expect(screen.getByText(CatalogFixtures.noResultsLine)).toBeInTheDocument();

        await fireEvent.click(screen.getByRole("button", { name: "Search every registry" }));
        await fireEvent.click(screen.getByRole("button", { name: "Clear search" }));

        expect(onsearchall).toHaveBeenCalledOnce();
        expect(onclearsearch).toHaveBeenCalledOnce();
    });

    it("leaves out Search every registry when every registry was searched", () => {
        render(AddModBrowseScreen, { props: props({ source: "all", groups: [], query: "dynmap", otherMatches: 4 }) });

        expect(screen.getByRole("heading", { name: "No mods match “dynmap”." })).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: "Search every registry" })).toBeNull();
    });

    it("keeps the selection bar on screen in every state, with Install disabled while nothing is ticked", () => {
        render(AddModBrowseScreen, { props: props({ source: "old", groups: [] }) });

        expect(screen.getByText("Tick mods to install several at once.")).toBeInTheDocument();
        expect(screen.getByRole("button", { name: "Install" })).toBeDisabled();
    });

    it("asks for the new ticked ids and for the mod to open, and draws neither itself", async () => {
        const onchecked = vi.fn();
        const onactivate = vi.fn();
        const groups = [{ ...CatalogFixtures.browseGroups()[0]!, offset: 0, total: 20 }];
        render(AddModBrowseScreen, { props: props({ groups, checked: [], onchecked, onactivate }) });
        const box = screen.getByRole("checkbox", { name: "Select Chat Heads" });

        await fireEvent.click(box);
        await fireEvent.click(screen.getByRole("button", { name: /Chunk Borders/ }));

        expect(onchecked).toHaveBeenCalledExactlyOnceWith(["@mira/chatheads"]);
        expect(box).toHaveAttribute("aria-checked", "false");
        expect(onactivate).toHaveBeenCalledExactlyOnceWith("@kv/chunkborders");
    });
});
