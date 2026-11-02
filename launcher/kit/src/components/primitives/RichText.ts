import type { TextPart } from "../../text/Template";

export interface RichTextProps {
    /** The runs from `Template.parts()`. A strong run is drawn in a `<strong>` element. */
    parts: TextPart[];
}
