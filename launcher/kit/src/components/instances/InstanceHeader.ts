import type { Instance } from "../../model";
import { Template } from "../../text/Template";

export interface InstanceHeaderText {
    /** Back link label ("Instances"). */
    back: string;
    delete: string;
    openFolder: string;
    launch: string;
    running: string;
    /** Template with `{game}`, `{data}` and `{folder}` slots, for example "Game {game} · {data} · {folder}". */
    detail: string;
    dataShared: string;
    dataOwn: string;
}

export interface InstanceHeaderProps {
    instance: Instance;
    text: InstanceHeaderText;
    onback?: () => void;
    ondelete?: (id: string) => void;
    onopenfolder?: (id: string) => void;
    /** Not called for a running instance: the header shows a status in place of the Launch button. */
    onlaunch?: (id: string) => void;
}

export class InstanceHeaderRules {
    /** "Game 1.26.60.5 · shared data · %LOCALAPPDATA%\amml\instances\1.26.60.05". */
    static detail(instance: Instance, text: InstanceHeaderText): string {
        return Template.fill(text.detail, {
            game: instance.gameVersion,
            data: instance.dataMode === "shared" ? text.dataShared : text.dataOwn,
            folder: instance.folder,
        });
    }
}
