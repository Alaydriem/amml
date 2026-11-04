import type { AddModContext, FoundMod, RegistrySource, UrlCheck } from "../model";
import type { AddModHeaderText } from "../components/catalog/AddModHeader";
import type { SourceNavText } from "../components/catalog/SourceNav";
import type { UrlInstallPaneText } from "../components/catalog/UrlInstallPane";
import { Template, type TextPart } from "../text/Template";

export interface AddModUrlScreenText {
    header: AddModHeaderText;
    nav: SourceNavText;
    pane: UrlInstallPaneText;
    /** Template with `{label}` and `{id}` slots; `**…**` marks the strong part: "**{label}** · {id}". */
    found: string;
    /** Caps warn button of the bottom bar. */
    installUnofficial: string;
}

export interface AddModUrlScreenProps {
    /** Name of the instance the mod goes to, shown strong in the title. */
    name: string;
    context: AddModContext;
    registries: RegistrySource[];
    allCount?: number;
    showOtherAmml?: boolean;
    /** Shown link. The screen never changes it: the app answers `oninput` with a new value. */
    url?: string;
    checks: UrlCheck[];
    /** Shows the warn callout; leave it off when the mod is in a registry. */
    unofficial?: boolean;
    checking?: boolean;
    /** The mod the checks found. The bottom bar shows when it is set. */
    found?: FoundMod;
    text: AddModUrlScreenText;
    onback?: () => void;
    /** A source other than "From a URL" was chosen. */
    onsource?: (id: string) => void;
    onshowotheramml?: (show: boolean) => void;
    oninput?: (url: string) => void;
    oncheck?: (url: string) => void;
    oninstall?: () => void;
}

export class AddModUrlScreenRules {
    static found(found: FoundMod, text: AddModUrlScreenText): TextPart[] {
        return Template.parts(text.found, { label: found.label, id: found.id });
    }
}
