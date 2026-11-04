/** The links of the About page, in order (artboard 07e). Their words are in `en.aboutSettings`. */
export type AboutLinkId = "source" | "issues" | "licence" | "notices";

/** One link row of the About page. */
export interface AboutLink {
    id: AboutLinkId;
    /** Address passed to `onopenlink`. */
    href: string;
}
