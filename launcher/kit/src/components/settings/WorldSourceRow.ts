export interface WorldSourceRowText {
    copy: string;
}

export interface WorldSourceRowProps {
    id: string;
    name: string;
    /** Mono second line, for example `%APPDATA%\Minecraft Bedrock · 3 worlds · 5 mo ago`. */
    detail: string;
    text: WorldSourceRowText;
    oncopy?: (id: string) => void;
}
