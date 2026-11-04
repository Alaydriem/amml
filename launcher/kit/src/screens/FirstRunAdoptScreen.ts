import type { AdoptedInstance, ChoiceItem } from "../model";
import type { AdoptedInstanceText } from "../components/setup/AdoptedInstanceRow";

export interface FirstRunAdoptScreenText {
    eyebrow: string;
    /** Title with the bold part marked by `**…**`, for example "Keep your instances. **Share your worlds.**". */
    title: string;
    intro: string;
    /** Panel header of the found instances. */
    foundHeading: string;
    /** Panel header of the data choices. */
    copyHeading: string;
    adopted: AdoptedInstanceText;
    footerNote: string;
    back: string;
    next: string;
}

export interface FirstRunAdoptScreenProps {
    /** Instances found on disk, adopted as they are. */
    instances: AdoptedInstance[];
    /** Where the shared data comes from. */
    choices: ChoiceItem[];
    /** Id of the chosen source. The screen never changes it: the parent answers `onselect` with a new value. */
    selected?: string;
    text: FirstRunAdoptScreenText;
    onselect?: (id: string) => void;
    onnext?: () => void;
    onback?: () => void;
}
