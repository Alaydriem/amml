import type { TextTone } from "../../model";

export interface RegistryRowText {
    /** Tag of a registry that ships with the launcher; it cannot be removed. */
    builtIn: string;
    remove: string;
}

export interface RegistryRowProps {
    id: string;
    name: string;
    /** Registry URL as shown, mono. */
    url: string;
    /** Right-hand status text ("12 mods · 4 min ago", "Fetch failed"). */
    status: string;
    statusTone?: Extract<TextTone, "ok" | "fault" | "faint">;
    /** A built-in registry shows the `builtIn` tag and cannot be removed. */
    builtIn?: boolean;
    text: RegistryRowText;
    onremove?: (id: string) => void;
}
