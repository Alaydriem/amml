// BSD-3-Clause. Copyright (c) Charles R. Portwood II. Adapted from the Bedrock Voice Chat client.

/**
 * Which files hold shippable interface copy.
 *
 * Shared by the extractor and any coverage check so the two cannot disagree about what the denominator is.
 */
export default class Sources {
    static readonly GLOB = "src/**/*.{ts,svelte}";

    static readonly IGNORED: readonly string[] = [
        // Generated from the Rust types; the generator is what is under test, not its output.
        "src/lib/bindings/",
        // The localization modules hold marker names, not copy.
        "src/lib/i18n/",
        // Test files and test helpers.
        "tests/",
    ];

    static isIgnored(path: string): boolean {
        const normalized = path.split("\\").join("/");
        return Sources.IGNORED.some((prefix) => normalized.includes(prefix));
    }

    /**
     * Names that stay English wherever they appear. Translating a product or a platform produces text that matches
     * nothing the reader can find, in the game, on their device or in a search.
     */
    static readonly PROPER: ReadonlySet<string> = new Set([
        "AMML",
        "Minecraft",
        "Minecraft Bedrock",
        "Microsoft Store",
        "Xbox",
        "GameInput",
        "Gaming Services",
        "Windows",
        "GitHub",
    ]);

    /**
     * Key names as they are printed on the key. `Num Enter` is what the keycap says, so translating it names a key
     * nobody has.
     */
    static readonly KEY_NAME = /^(?:Num|Numpad|Fn|Alt|Ctrl|Shift|Meta|Cmd|Win)\b[\s\S]{0,12}$/;

    /** True when every token is a proper noun, such as `Windows · GitHub`. Version numbers count as part of a name. */
    static isProperNoun(text: string): boolean {
        const trimmed = text.trim().replace(/\s+/g, " ");
        if (Sources.PROPER.has(trimmed)) return true;
        if (Sources.KEY_NAME.test(trimmed)) return true;

        const tokens = trimmed.split(/[\s·+&,—–]+/).filter(Boolean);
        return (
            tokens.length > 0 &&
            tokens.every((token) => Sources.PROPER.has(token) || /^v?\d+(\.\d+)*$/.test(token))
        );
    }

    /**
     * Log calls. Their arguments reach a file, never a reader, and marking them would put internal text in front of
     * a translator.
     */
    static readonly LOG_CALL = /\b(?:warn|info|debug|error|trace|log)\s*\(\s*$/;
}
