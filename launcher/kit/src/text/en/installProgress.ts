import type { InstallProgressScreenText } from "../../screens/InstallProgressScreen";

export const installProgressEn: InstallProgressScreenText = {
    back: "Instances",
    title: "Creating **{name}**",
    failedTitle: "Could not create **{name}**",
    stepsLabel: "Install steps",
    steps: {
        done: "Done",
        active: "In progress",
        waiting: "Waiting",
        failed: "Failed",
    },
    step: "Step {current} of {total}",
    stopped: "Stopped at step {current} of {total}",
    readout: {
        progressLabel: "Download progress",
        timeLeft: "about {time} left",
        separator: " · ",
    },
    warning:
        "**AMML is creating your new instance. Do not start Minecraft from the Store during this install.** This may take a minute or two to complete.",
    restored: "**Your Store version was put back.** It starts as before. No instance folder was made.",
    cancel: "Cancel install",
    retry: "Try again",
    showLog: "Show log",
};
