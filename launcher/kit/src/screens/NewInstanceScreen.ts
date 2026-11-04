import type { Channel, DataMode, GameVersion, NewInstanceSummary, NewInstanceSubmit } from "../model";
import type { NewInstanceFormText } from "../components/instances/NewInstanceForm";
import type { VersionRowText } from "../components/instances/VersionRow";
import type { Plural } from "../text/Template";

export interface NewInstanceScreenText {
    /** Back link label ("Instances"). */
    back: string;
    /** Title with the bold part marked by `**…**`, for example "Create a new **instance**". */
    title: string;
    versionLabel: string;
    release: string;
    preview: string;
    /** Accessible name of the version list. */
    versionsLabel: string;
    versionRow: VersionRowText;
    form: NewInstanceFormText;
    /** Template with a `{name}` slot, for example "instances\{name}". */
    folderPath: string;
    /** Template with `{amml}` and `{mods}` slots, for example "Installs **{amml}** and **{mods}**.". */
    installs: string;
    /** Template with a `{version}` slot, for example "AMML {version}". */
    amml: string;
    /** "{count} default mod" and "{count} default mods". */
    mods: Plural;
    /** Template with a `{size}` slot, for example "About {size} on disk." */
    disk: string;
}

export interface NewInstanceScreenProps {
    versions: GameVersion[];
    /** Names of existing instances. */
    takenNames: string[];
    summary: NewInstanceSummary;
    text: NewInstanceScreenText;
    channel: Channel;
    /** Why the app refuses the name, in the user's language. It shows under the name field and holds Create back. */
    problem?: string;
    /** Shows the release and preview switch. Default true; the app turns it off when preview builds are off. */
    showPreview?: boolean;
    /** Version string of the chosen row. */
    selectedVersion?: string;
    name: string;
    dataMode: DataMode;
    onchannelchange?: (channel: Channel) => void;
    onversionchange?: (version: string) => void;
    onnamechange?: (name: string) => void;
    ondatamodechange?: (dataMode: DataMode) => void;
    onback?: () => void;
    oncancel?: () => void;
    oncreate?: (submit: NewInstanceSubmit) => void;
}

export class NewInstanceScreenRules {
    /** The versions of one channel, in the order the app gave them. */
    static visible(versions: GameVersion[], channel: Channel): GameVersion[] {
        return versions.filter((version) => version.channel === channel);
    }

    /** True when the chosen version is in the list, in this channel, and the build is supported. */
    static selectable(versions: GameVersion[], channel: Channel, selected: string | undefined): boolean {
        return NewInstanceScreenRules.visible(versions, channel).some(
            (version) => version.version === selected && version.state !== "unsupported",
        );
    }
}
