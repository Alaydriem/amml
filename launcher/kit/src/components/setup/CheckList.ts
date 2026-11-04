import type { SystemCheck } from "../../model";
import type { CheckRowText } from "./CheckRow";

export interface CheckListProps {
    /** Panel header ("This PC"). */
    heading: string;
    checks: SystemCheck[];
    text: CheckRowText;
    onaction?: (id: string) => void;
}
