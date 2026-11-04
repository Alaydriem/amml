/** The gallery page that each section of the settings nav opens. */
export class SettingsPages {
    private static readonly SLUGS: Record<string, string> = {
        registries: "s-07a-registries",
        versions: "s-07b-game-versions",
        shared: "s-07c-shared-data",
        downloads: "s-07d-downloads",
        about: "s-07e-about",
    };

    static open(id: string): void {
        const slug = SettingsPages.SLUGS[id];
        if (slug) location.hash = slug;
    }
}
