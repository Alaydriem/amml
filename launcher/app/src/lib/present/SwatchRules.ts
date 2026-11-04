import type { Swatch } from "@amml/launcher-kit";

// The colour of an instance or mod tile: the same name always gives the same one of the ten.
export class SwatchRules {
    private static readonly OFFSET_BASIS = 0x811c9dc5;
    private static readonly PRIME = 0x01000193;

    static forName(name: string): Swatch {
        let hash = SwatchRules.OFFSET_BASIS;
        for (const byte of new TextEncoder().encode(name)) {
            hash ^= byte;
            hash = Math.imul(hash, SwatchRules.PRIME) >>> 0;
        }
        return ((hash % 10) + 1) as Swatch;
    }
}
