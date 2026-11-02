/** The instance the add-mod screens add to, as the context chip shows it. */
export interface AddModContext {
    /** Game version, for example `1.26.60.5`. */
    game: string;
    /** AMML version, for example `26.60.5`. */
    amml: string;
    /** Mod api number, for example `2`. */
    api: string;
}
