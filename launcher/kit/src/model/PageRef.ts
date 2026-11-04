/**
 * A page of one registry's index that the result list asks the app to load.
 *
 * `page` is the registry index page, counted from 1: the page that holds index position `i` is
 * `floor(i / pageSize) + 1`. Groups of one registry share its pages, so `registryId` and `page` name a page
 * on their own.
 */
export interface PageRef {
    registryId: string;
    page: number;
}
