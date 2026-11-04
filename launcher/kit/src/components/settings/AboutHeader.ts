/** The fixed words of the About header. */
export interface AboutHeaderText {
    /** The title, with the bold part in `**…**` ("AMML **Launcher**"). */
    title: string;
    /** Accessible name of the brand mark. */
    markLabel: string;
    /** Notice above the update button; `{version}` is the newer version. */
    updateNotice: string;
    update: string;
}

export interface AboutHeaderProps {
    text: AboutHeaderText;
    /** Mono build line, already built from the version, the date and the platform. */
    build: string;
    /** Newest version when it is newer than the running one. The notice and the button show only when it is set. */
    updateVersion?: string;
    onupdate?: () => void;
}
