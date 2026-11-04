/** A folder the user can copy worlds from (artboards 02, 07c). */
export interface WorldSource {
    id: string;
    name: string;
    folder: string;
    worldCount: number;
    /** Free text the app formats, for example "5 mo ago". */
    modified: string;
}
