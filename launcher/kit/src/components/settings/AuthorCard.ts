import type { AboutAuthor } from "../../model";

export interface AuthorCardText {
    /** Line next to the avatar; `{name}` is the author's name ("Made by {name}"). */
    byline: string;
    /** Label of the button that opens the author's page. */
    open: string;
}

export interface AuthorCardProps {
    author: AboutAuthor;
    text: AuthorCardText;
    /** Called with the author's address when the button is pressed. */
    onopenlink?: (href: string) => void;
}
