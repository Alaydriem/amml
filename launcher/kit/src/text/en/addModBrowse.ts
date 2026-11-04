import type { AddModBrowseScreenText } from "../../screens/AddModBrowseScreen";
import { shellEn } from "./shell";

export const addModBrowseEn: AddModBrowseScreenText = {
    header: {
        back: "Back",
        title: "Add mods to **{name}**",
        labels: { game: "Game", amml: "AMML", api: "api" },
        chip: shellEn.contextChip,
    },
    toolbar: {
        searchLabel: "Search mods",
        sortLabel: "Sort by",
        sortOptions: [
            { id: "name", label: "Name" },
            { id: "updated", label: "Recently updated" },
            { id: "size", label: "Size" },
        ],
    },
    searchIn: "Search {name}",
    searchAll: "Search all registries",
    nav: {
        heading: "Source",
        allRegistries: "All registries",
        fromUrl: "From a URL",
        offline: "offline",
        failed: "failed",
        needOtherAmml: "Show mods that need another AMML",
    },
    list: {
        listLabel: "Mods",
        installed: "Installed",
        update: "Update",
        selectRow: "Select {title}",
        loadingRange: "loading {from}–{to}",
        loadingStatus: "Loading more mods",
        groupCount: "{name} · {count}",
        more: "+{count} more in {name}",
    },
    detail: {
        versionFor: "Version for api {api}",
        download: "Download",
        source: "Source",
        downloadValue: "{size} · {checksum}",
        checksum: "checksum",
        noChecksum: "no checksum",
        subline: "{id} · {registry}",
        note: "Details load when you select a mod. Older versions are on the source page.",
        loading: "loading {document}",
        empty: "Select a mod to see what it does, who made it and how big the download is.",
    },
    bar: {
        summary: "**{count} selected** · {names}",
        nameSeparator: ", ",
        clear: "Clear",
        install: { one: "Install {count} mod", other: "Install {count} mods" },
        installNone: "Install",
        hint: "Tick mods to install several at once.",
    },
    offline: {
        title: "{name} is offline.",
        body: "The launcher could not reach **{url}** and has no saved copy of it. Mods from your other registries still install.",
        retry: "Try again",
        settings: "Registry settings",
        line: "No list · last tried {fetched}",
        lineNever: "No list",
    },
    noResults: {
        title: "No mods in {name} match “{query}”.",
        titleAll: "No mods match “{query}”.",
        body: {
            one: "Search looks at names, ids and tags, not descriptions. {count} mod in your other registries matches.",
            other: "Search looks at names, ids and tags, not descriptions. {count} mods in your other registries match.",
        },
        bodyNone: "Search looks at names, ids and tags, not descriptions.",
        searchAll: "Search every registry",
        clear: "Clear search",
    },
};
