import type { LauncherApi } from "#lib/api/LauncherApi.ts";
import type { FaultView } from "#lib/bindings/FaultView.ts";
import { StoreLinks } from "#lib/present/StoreLinks.ts";
import { I18n } from "#lib/i18n/index.ts";
import type { Navigate } from "./Navigate.ts";
import type { ShellStore } from "./ShellStore.svelte.ts";

export type FaultActionsApi = Pick<typeof LauncherApi, "openUrl">;

// What the action button of a fault banner does, by the kind of the fault.
export class FaultActions {
    private readonly shell: Pick<ShellStore, "showLog" | "dismissStartupFault" | "say">;
    private readonly api: FaultActionsApi;
    private readonly navigate: Navigate;
    private readonly copy: (text: string) => Promise<void>;

    constructor(
        shell: Pick<ShellStore, "showLog" | "dismissStartupFault" | "say">,
        api: FaultActionsApi,
        navigate: Navigate,
        copy: (text: string) => Promise<void>,
    ) {
        this.shell = shell;
        this.api = api;
        this.navigate = navigate;
        this.copy = copy;
    }

    // `retry` runs for the faults whose action is "try again"
    async run(fault: FaultView, retry?: () => void | Promise<void>): Promise<void> {
        switch (fault.kind) {
            case "store-missing":
                await this.api.openUrl(StoreLinks.minecraft);
                return;
            case "gaming-services-missing":
                await this.api.openUrl(StoreLinks.gamingServices);
                return;
            case "preview-missing":
                await this.api.openUrl(StoreLinks.preview);
                return;
            case "install-interrupted":
                await this.shell.showLog();
                this.shell.dismissStartupFault();
                return;
            case "operation-failed":
                await this.shell.showLog();
                return;
            case "busy":
                this.navigate("/instances/install");
                return;
            case "restore-failed":
            case "deploy-refused":
                await this.copy(fault.code ?? "");
                this.shell.say(I18n.t("Copied"));
                return;
            case "consent-needed":
                return;
            default:
                await retry?.();
        }
    }
}
