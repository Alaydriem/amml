import type { Option } from "../../model";
import type { ContextChipText } from "../../components/primitives/ContextChip";
import type { SettingsNavText } from "../../components/shell/SettingsNav";
import type { StatusBarText } from "../../components/shell/StatusBar";
import type { TitleBarText } from "../../components/shell/TitleBar";

/** The text of the parts every screen shares: the title bar, the status bar, the settings nav and the context chip. */
export interface ShellText {
    titleBar: TitleBarText;
    statusBar: StatusBarText;
    settingsNav: SettingsNavText;
    /** The sections of the settings nav, in order. */
    settingsSections: Option[];
    contextChip: ContextChipText;
}

export const shellEn: ShellText = {
    titleBar: {
        wordmark: "AMML Launcher",
        refresh: "Refresh registries",
        settings: "Settings",
        step: "Setup · {current} of {total}",
        registry: "Registry · {fetched}",
    },
    statusBar: {
        left: "launcher {version} · data {dataPath}",
        store: "Store Bedrock {version} registered",
    },
    settingsNav: {
        back: "Instances",
        label: "Settings sections",
    },
    settingsSections: [
        { id: "registries", label: "Registries" },
        { id: "versions", label: "Game versions" },
        { id: "shared", label: "Shared data" },
        { id: "downloads", label: "Downloads" },
        { id: "about", label: "About" },
    ],
    contextChip: {
        separator: "·",
    },
};
