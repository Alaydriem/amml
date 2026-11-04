import type { RegistrySource } from "../../model";
import { Units } from "../../text/Units";

export interface SourceNavText {
    /** The mono caps heading ("Source"). */
    heading: string;
    allRegistries: string;
    fromUrl: string;
    offline: string;
    failed: string;
    /** The toggle at the foot: "Show mods that need another AMML". */
    needOtherAmml: string;
}

export interface SourceNavProps {
    registries: RegistrySource[];
    /** Count beside "All registries". */
    allCount?: number;
    /** Id of the chosen item: a registry id, `SourceNavRules.ALL` or `SourceNavRules.URL`. */
    value?: string;
    /** The "need another AMML" toggle. */
    showOtherAmml?: boolean;
    text: SourceNavText;
    /** Called with the id of the item the user picked. */
    onchange?: (id: string) => void;
    /** Called with the state the user asks for. The toggle shows only when this is set. */
    onshowotheramml?: (show: boolean) => void;
}

export class SourceNavRules {
    /** Id of the "All registries" item. */
    static readonly ALL = "all";
    /** Id of the "From a URL" item. */
    static readonly URL = "url";

    /** The right-hand text of a registry item: its mod count, or the state when it has no list to count. */
    static trailing(registry: RegistrySource, text: SourceNavText): { text: string; fault: boolean } {
        if (registry.status === "offline") return { text: text.offline, fault: true };
        if (registry.status === "failed") return { text: text.failed, fault: true };
        return { text: registry.modCount === undefined ? "" : Units.en.count(registry.modCount), fault: false };
    }
}
