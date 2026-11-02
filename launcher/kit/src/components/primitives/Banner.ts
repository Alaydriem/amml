import type { Snippet } from "svelte";
import type { Severity } from "../../model";
import type { IconName } from "./Icon";

export interface BannerProps {
    /** `fault` and `warn` rules; `brand` and `info` share the brand-lift rule. */
    tone?: Severity;
    /** Leading icon. Without it the banner draws the icon of its tone. */
    icon?: IconName;
    /** The message. Wrap the lead sentence in `<strong>`; wrap codes and sizes in `<code>`. */
    children: Snippet;
    /** Trailing control, usually a `Button`. */
    action?: Snippet;
}

export class BannerRules {
    static icon(tone: Severity): IconName {
        switch (tone) {
            case "fault":
            case "muted":
            case "ok":
                return "alert-circle";
            case "warn":
                return "alert-triangle";
            case "brand":
                return "arrow-up";
            case "info":
                return "refresh";
        }
    }

    /** `fault` is announced at once; the others wait for a pause. */
    static role(tone: Severity): "alert" | "status" {
        return tone === "fault" ? "alert" : "status";
    }
}
