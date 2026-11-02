import type { RegistryRowProps, RegistryRowText } from "../components/settings";
import type { RegistrySource } from "../model";
import { Template, type Plural } from "../text";

export interface RegistriesSettingsScreenText {
    heading: string;
    /** Link in the card header that adds a registry. */
    add: string;
    row: RegistryRowText;
    /** Status of a registry that answered, with its mod count; `{count}` and `{fetched}`. */
    statusOk: Plural;
    /** Status of a registry that answered, count unknown; `{fetched}`. */
    statusOkNoCount: string;
    statusFailed: string;
    statusOffline: string;
    statusLoading: string;
    note: string;
}

export interface RegistriesSettingsScreenProps {
    registries: RegistrySource[];
    text: RegistriesSettingsScreenText;
    onaddregistry?: () => void;
    onremoveregistry?: (id: string) => void;
}

export class RegistriesSettingsScreenRules {
    /** Status text and tone of one registry row. */
    static status(
        registry: RegistrySource,
        text: RegistriesSettingsScreenText,
    ): { text: string; tone: NonNullable<RegistryRowProps["statusTone"]> } {
        const fetched = registry.fetched ?? "";
        switch (registry.status) {
            case "ok":
                return {
                    text:
                        registry.modCount === undefined
                            ? Template.fill(text.statusOkNoCount, { fetched })
                            : Template.plural(text.statusOk, registry.modCount, { fetched }),
                    tone: "ok",
                };
            case "failed":
                return { text: text.statusFailed, tone: "fault" };
            case "offline":
                return { text: text.statusOffline, tone: "faint" };
            case "loading":
                return { text: text.statusLoading, tone: "faint" };
        }
    }
}
