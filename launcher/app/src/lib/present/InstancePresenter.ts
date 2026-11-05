import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import { ActivityTextSet } from "#lib/text/ActivityTextSet.ts";
import type { Instance } from "@amml/launcher-kit";
import { SwatchRules } from "./SwatchRules.ts";

// Turns an instance's facts into the kit's `Instance` model.
export class InstancePresenter {
    private static readonly LOCAL_APP_DATA = /^[A-Za-z]:\\Users\\[^\\]+\\AppData\\Local/i;

    // `text.unknownVersion` is shown when the instance's AMML version cannot be read
    static toInstance(view: InstanceView, text: { unknownVersion: string }, now: number): Instance {
        return {
            id: view.name,
            name: view.name,
            swatch: SwatchRules.forName(view.name),
            gameVersion: view.game,
            ammlVersion: view.ammlVersion ?? text.unknownVersion,
            ammlLocal: view.ammlLocal,
            modCount: view.modCount,
            dataMode: view.dataMode,
            folder: InstancePresenter.shownFolder(view.folder),
            activity: ActivityTextSet.line(view, now),
            running: view.runningSince !== null,
            updateCount: view.updateCount,
        };
    }

    // `C:\Users\me\AppData\Local\amml\instances\x` shows as `%LOCALAPPDATA%\amml\instances\x`
    static shownFolder(folder: string): string {
        return folder.replace(InstancePresenter.LOCAL_APP_DATA, "%LOCALAPPDATA%");
    }
}
