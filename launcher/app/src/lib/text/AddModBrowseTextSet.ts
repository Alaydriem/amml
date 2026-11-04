import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";
import { ShellTextSet } from "./ShellTextSet.ts";

// The text of the add a mod browse screen, with every string through I18n. Without a pack it equals the kit English.
export class AddModBrowseTextSet {
    static build(): LauncherText["addModBrowse"] {
        return {
            header: {
                back: I18n.t("Back"),
                title: I18n.t("Add mods to **{name}**"),
                labels: {
                    game: I18n.t("Game"),
                    amml: I18n.t("AMML"),
                    api: I18n.t("api"),
                },
                chip: ShellTextSet.build().contextChip,
            },
            toolbar: {
                searchLabel: I18n.t("Search mods"),
                sortLabel: I18n.t("Sort by"),
                sortOptions: [
                    {
                        id: "name",
                        label: I18n.t("Name"),
                    },
                    {
                        id: "updated",
                        label: I18n.t("Recently updated"),
                    },
                    {
                        id: "size",
                        label: I18n.t("Size"),
                    },
                ],
            },
            searchIn: I18n.t("Search {name}"),
            searchAll: I18n.t("Search all registries"),
            nav: {
                heading: I18n.t("Source"),
                allRegistries: I18n.t("All registries"),
                fromUrl: I18n.t("From a URL"),
                offline: I18n.t("offline"),
                failed: I18n.t("failed"),
                needOtherAmml: I18n.t("Show mods that need another AMML"),
            },
            list: {
                listLabel: I18n.t("Mods"),
                installed: I18n.t("Installed"),
                update: I18n.t("Update"),
                selectRow: I18n.t("Select {title}"),
                loadingRange: I18n.t("loading {from}–{to}"),
                loadingStatus: I18n.t("Loading more mods"),
                groupCount: I18n.t("{name} · {count}"),
                more: I18n.t("+{count} more in {name}"),
            },
            detail: {
                versionFor: I18n.t("Version for api {api}"),
                download: I18n.t("Download"),
                source: I18n.t("Source"),
                downloadValue: I18n.t("{size} · {checksum}"),
                checksum: I18n.t("checksum"),
                noChecksum: I18n.t("no checksum"),
                subline: I18n.t("{id} · {registry}"),
                note: I18n.t("Details load when you select a mod. Older versions are on the source page."),
                loading: I18n.t("loading {document}"),
                empty: I18n.t("Select a mod to see what it does, who made it and how big the download is."),
            },
            bar: {
                summary: I18n.t("**{count} selected** · {names}"),
                nameSeparator: I18n.tc("separator", ", "),
                clear: I18n.t("Clear"),
                install: {
                    one: I18n.tn("Install {count} mod", "Install {count} mods", 1),
                    other: I18n.tn("Install {count} mod", "Install {count} mods", 2),
                },
                installNone: I18n.t("Install"),
                hint: I18n.t("Tick mods to install several at once."),
            },
            offline: {
                title: I18n.t("{name} is offline."),
                body: I18n.t("The launcher could not reach **{url}** and has no saved copy of it. Mods from your other registries still install."),
                retry: I18n.t("Try again"),
                settings: I18n.t("Registry settings"),
                line: I18n.t("No list · last tried {fetched}"),
                lineNever: I18n.t("No list"),
            },
            noResults: {
                title: I18n.t("No mods in {name} match “{query}”."),
                titleAll: I18n.t("No mods match “{query}”."),
                body: {
                    one: I18n.tn("Search looks at names, ids and tags, not descriptions. {count} mod in your other registries matches.", "Search looks at names, ids and tags, not descriptions. {count} mods in your other registries match.", 1),
                    other: I18n.tn("Search looks at names, ids and tags, not descriptions. {count} mod in your other registries matches.", "Search looks at names, ids and tags, not descriptions. {count} mods in your other registries match.", 2),
                },
                bodyNone: I18n.t("Search looks at names, ids and tags, not descriptions."),
                searchAll: I18n.t("Search every registry"),
                clear: I18n.t("Clear search"),
            },
        };
    }
}
