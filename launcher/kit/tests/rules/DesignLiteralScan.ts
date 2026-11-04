/// <reference types="node" />
import { readFileSync } from "node:fs";

const tokensCss = readFileSync("src/css/tokens.css", "utf8");

const SOURCES = import.meta.glob<string>("../../src/**/*.svelte", { query: "?raw", import: "default", eager: true });

export interface LiteralIssue {
    /** Path of the file under `src/`, with forward slashes. */
    file: string;
    /** The rule that failed: `hex colour`, `named colour`, `font size`, `letter spacing`, `font weight`, `radius`, `opacity` or `undefined token`. */
    kind: string;
    /** The declaration or token that failed. */
    detail: string;
}

interface Declaration {
    property: string;
    value: string;
}

/**
 * Finds design values written as literals in the kit's Svelte files. A colour, font size, letter spacing, font
 * weight, radius or opacity must come from a `var(--…)` token, and every `var(--name)` must name a token that
 * `tokens.css` or the same file defines. Padding, gap, width, height and line height may stay literal.
 */
export class DesignLiteralScan {
    private static readonly COLOUR_PROPERTIES =
        /^(color|background|background-color|border|border-(top|right|bottom|left)(-color)?|border-color|outline|outline-color|fill|stroke|box-shadow|text-shadow|accent-color|caret-color)$/;

    private static readonly NAMED_COLOUR =
        /\b(white|black|red|green|blue|yellow|orange|purple|pink|gray|grey|silver|gold|cyan|magenta|brown|navy|teal|lime|maroon|olive|aqua|fuchsia)\b/i;

    private static tokenCache: Set<string> | undefined;

    /** Names of the tokens defined in `tokens.css`. */
    static tokens(): Set<string> {
        if (!DesignLiteralScan.tokenCache) {
            DesignLiteralScan.tokenCache = new Set(
                [...tokensCss.matchAll(/^\s*(--[\w-]+)\s*:/gm)].map((match) => match[1] ?? ""),
            );
        }
        return DesignLiteralScan.tokenCache;
    }

    /** Scans every `.svelte` file under a folder of `src/` and returns one issue for each distinct file and finding. */
    static scanFolder(folder: string): LiteralIssue[] {
        const prefix = `../../src/${folder}/`;
        return Object.entries(SOURCES)
            .filter(([path]) => path.startsWith(prefix))
            .flatMap(([path, source]) => DesignLiteralScan.scanSource(path.slice("../../src/".length), source));
    }

    /** Scans one source text. `file` only labels the issues. */
    static scanSource(file: string, source: string): LiteralIssue[] {
        const found = new Map<string, LiteralIssue>();
        const add = (kind: string, detail: string) => {
            const key = `${kind}|${detail}`;
            if (!found.has(key)) found.set(key, { file, kind, detail });
        };
        for (const declaration of DesignLiteralScan.declarations(source)) {
            DesignLiteralScan.checkDeclaration(declaration, add);
        }
        const local = new Set([...source.matchAll(/(--[\w-]+)\s*:/g)].map((match) => match[1] ?? ""));
        for (const match of source.matchAll(/var\(\s*(--[\w-]+)/g)) {
            const name = match[1] ?? "";
            if (!DesignLiteralScan.tokens().has(name) && !local.has(name)) add("undefined token", name);
        }
        return [...found.values()];
    }

    private static checkDeclaration({ property, value }: Declaration, add: (kind: string, detail: string) => void): void {
        const bare = DesignLiteralScan.withoutVars(value).trim();
        const detail = `${property}: ${value.trim()}`;
        if (/#[0-9a-fA-F]{3,8}\b/.test(bare)) add("hex colour", detail);
        if (/\b(rgba?|hsla?|oklch|oklab|hwb|lab|lch)\(/i.test(bare)) add("hex colour", detail);
        if (DesignLiteralScan.COLOUR_PROPERTIES.test(property) && DesignLiteralScan.NAMED_COLOUR.test(bare)) {
            add("named colour", detail);
        }
        if (property === "font-size" && /\d/.test(bare)) add("font size", detail);
        if (property === "letter-spacing" && /\d/.test(bare) && !/^0+(px|em)?$/.test(bare)) add("letter spacing", detail);
        if (property === "font-weight" && /^(\d+|bold|bolder|lighter|normal)$/.test(bare)) add("font weight", detail);
        if (/^border(-(top|bottom)-(left|right))?-radius$/.test(property) && /(\d+(\.\d+)?|\})(px|em|rem)\b/.test(bare)) {
            add("radius", detail);
        }
        if (property === "opacity" && bare !== "" && !/^(0|1|inherit)$/.test(bare)) add("opacity", detail);
    }

    private static declarations(source: string): Declaration[] {
        const result: Declaration[] = [];
        for (const block of source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)) {
            for (const match of (block[1] ?? "").matchAll(/(^|[\s;{])([a-z-]+)\s*:\s*([^;{}]+);/g)) {
                result.push({ property: match[2] ?? "", value: match[3] ?? "" });
            }
        }
        const markup = source.replace(/<style[^>]*>[\s\S]*?<\/style>/g, "").replace(/<script[^>]*>[\s\S]*?<\/script>/g, "");
        for (const match of markup.matchAll(/style:([a-z-]+)=("[^"]*"|\{[^}]*\})/g)) {
            result.push({ property: match[1] ?? "", value: match[2] ?? "" });
        }
        for (const match of markup.matchAll(/\sstyle="([^"]*)"/g)) {
            for (const part of (match[1] ?? "").split(";")) {
                const [property, ...rest] = part.split(":");
                if (property && rest.length > 0) result.push({ property: property.trim(), value: rest.join(":") });
            }
        }
        return result;
    }

    private static withoutVars(value: string): string {
        let text = value;
        let previous = "";
        while (text !== previous) {
            previous = text;
            text = text.replace(/var\([^()]*\)/g, "");
        }
        return text;
    }
}
