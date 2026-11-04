import type { DownloadsView } from "#lib/bindings/DownloadsView.ts";
import { DownloadsDetailTextSet } from "#lib/text/DownloadsDetailTextSet.ts";
import type { DownloadFile } from "@amml/launcher-kit";

// Turns the downloads folder's facts into the kit's file rows.
export class DownloadsPresenter {
    static toFiles(view: DownloadsView, now: number): DownloadFile[] {
        return view.files.map((file) => ({
            id: file.id,
            name: file.id,
            size: file.size,
            state: file.state === "finished" ? "waiting" : file.state,
            detail: DownloadsDetailTextSet.line(file, now),
        }));
    }
}
