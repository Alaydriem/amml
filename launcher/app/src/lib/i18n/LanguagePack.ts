// BSD-3-Clause. Copyright (c) Charles R. Portwood II.

// One translated message: a single string, or one string per plural form.
export type PackEntry = string | string[];

// A compiled translation for one locale, as `amml-i18n` writes it. English is never a pack: message ids are the
// English text.
export interface LanguagePack {
    v: number;
    locale: string;
    // CLDR plural category names in form order
    plural: string[];
    m: Record<string, PackEntry>;
}
