import type { AboutSettingsScreenText } from "../../screens/AboutSettingsScreen";

export const aboutSettingsEn: AboutSettingsScreenText = {
    header: {
        title: "AMML **Launcher**",
        markLabel: "AMML Launcher",
        updateNotice: "{version} is out",
        update: "Update and restart",
    },
    build: "{version} · built {builtOn} · {platform}",
    author: {
        byline: "Made by {name}",
        open: "GitHub",
    },
    projectHeading: "Project",
    links: {
        source: { label: "Source code", value: "Alaydriem/amml" },
        issues: { label: "Report a problem", value: "GitHub issues" },
        licence: { label: "Licence", value: "BSD 3-Clause" },
        notices: { label: "Third-party notices", value: "Tauri, Svelte, fonts" },
    },
    bugReport: {
        heading: "For a bug report",
        copy: "Copy all",
        line: "{label}: {value}",
        linkValue: "{value} · Open",
        labels: {
            windows: "Windows",
            store: "Store Bedrock",
            devMode: "Developer Mode",
            registry: "Registry",
            logs: "Logs",
        },
    },
    disclaimer:
        "Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft. The game is installed from your own Store licence.",
};
