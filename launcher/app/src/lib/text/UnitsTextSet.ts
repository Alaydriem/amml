import { I18n } from "#lib/i18n/index.ts";
import type { LauncherText } from "@amml/launcher-kit";

// The text of sizes, rates and durations, with every string through I18n. Without a pack it equals the kit English.
export class UnitsTextSet {
    static build(): LauncherText["units"] {
        return {
            gigabytes: I18n.t("{n} GB"),
            megabytes: I18n.t("{n} MB"),
            kilobytes: I18n.t("{n} KB"),
            megabytesPerSecond: I18n.t("{n} MB/s"),
            seconds: I18n.t("{n} s"),
            minutes: I18n.t("{n} min"),
            hours: I18n.t("{n} h"),
            hoursMinutes: I18n.t("{hours} h {minutes} min"),
            locale: I18n.tc("number format", "en-US"),
        };
    }
}
