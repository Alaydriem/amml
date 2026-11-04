import type { DownloadFileView } from "#lib/bindings/DownloadFileView.ts";
import { I18n } from "#lib/i18n/index.ts";
import { ActivityTextSet } from "./ActivityTextSet.ts";

// The second line of a file in the downloads folder.
export class DownloadsDetailTextSet {
    static line(file: DownloadFileView, now: number): string {
        switch (file.state) {
            case "kept":
                return I18n.t("Your Store version · kept for the restore step");
            case "unfinished":
                return I18n.t("Unfinished · resumes on the next install");
            case "finished":
                return I18n.tf("Downloaded · {ago}", { ago: ActivityTextSet.ago(now - file.modified) });
        }
    }
}
