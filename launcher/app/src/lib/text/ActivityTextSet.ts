import type { InstanceView } from "#lib/bindings/InstanceView.ts";
import { I18n } from "#lib/i18n/index.ts";

// The activity line of an instance: running since, last played, or never played. Times are Unix milliseconds.
export class ActivityTextSet {
    private static readonly MINUTE = 60_000;
    private static readonly HOUR = 60 * ActivityTextSet.MINUTE;
    private static readonly DAY = 24 * ActivityTextSet.HOUR;
    private static readonly MONTH = 30 * ActivityTextSet.DAY;

    static line(view: InstanceView, now: number): string {
        if (view.runningSince !== null) {
            return I18n.tf("running since {time}", { time: ActivityTextSet.clock(view.runningSince) });
        }
        if (view.lastPlayed !== null) {
            return I18n.tf("last played {ago}", { ago: ActivityTextSet.ago(now - view.lastPlayed) });
        }
        return I18n.t("never played");
    }

    // "4 min ago", "2 h ago", "3 d ago", "5 mo ago" for a span in milliseconds
    static ago(elapsed: number): string {
        const span = Math.max(0, elapsed);
        if (span < ActivityTextSet.HOUR) {
            return I18n.tf("{n} min ago", { n: Math.max(1, Math.floor(span / ActivityTextSet.MINUTE)) });
        }
        if (span < ActivityTextSet.DAY) {
            return I18n.tf("{n} h ago", { n: Math.floor(span / ActivityTextSet.HOUR) });
        }
        if (span < ActivityTextSet.MONTH) {
            return I18n.tf("{n} d ago", { n: Math.floor(span / ActivityTextSet.DAY) });
        }
        return I18n.tf("{n} mo ago", { n: Math.floor(span / ActivityTextSet.MONTH) });
    }

    // local time as HH:mm
    private static clock(time: number): string {
        const date = new Date(time);
        const two = (n: number) => String(n).padStart(2, "0");
        return `${two(date.getHours())}:${two(date.getMinutes())}`;
    }
}
