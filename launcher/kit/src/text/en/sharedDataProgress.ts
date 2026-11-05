import type { InstallProgressScreenText } from "../../screens/InstallProgressScreen";
import { installProgressEn } from "./installProgress";

export const sharedDataProgressEn: InstallProgressScreenText = {
    ...installProgressEn,
    back: undefined,
    cancel: undefined,
    eyebrow: "Setting up shared data",
    title: "Copying worlds from **{name}**",
    failedTitle: "Could not copy worlds from **{name}**",
    stepsLabel: "Setup steps",
    readout: { ...installProgressEn.readout, progressLabel: "Copy progress" },
    warning:
        "**AMML is copying your worlds into shared data. Do not start Minecraft from the Store during this copy.** Nothing is moved or deleted: your Store worlds stay where they are.",
    restored:
        "**Nothing was changed.** Your Store worlds are where they were. The part copy was removed and no settings were saved.",
    other: "Choose another source",
};
