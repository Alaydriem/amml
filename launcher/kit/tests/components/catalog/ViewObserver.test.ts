import { afterEach, describe, expect, it, vi } from "vitest";
import { ViewObserver } from "../../../src/components/catalog/ViewObserver";

describe("ViewObserver", () => {
    afterEach(() => {
        vi.unstubAllGlobals();
    });

    it("looks one margin ahead above and below the container", () => {
        const root = document.createElement("div");

        expect(ViewObserver.options(root, 5700)).toEqual({ root, rootMargin: "5700px 0px" });
        expect(ViewObserver.options(null, -10)).toEqual({ root: null, rootMargin: "0px 0px" });
    });

    it("hands the margin to the browser observer and reports only elements that intersect", () => {
        const seen: IntersectionObserverInit[] = [];
        let report: IntersectionObserverCallback = () => {};
        vi.stubGlobal(
            "IntersectionObserver",
            class {
                constructor(callback: IntersectionObserverCallback, options: IntersectionObserverInit) {
                    report = callback;
                    seen.push(options);
                }
                observe() {}
                unobserve() {}
                disconnect() {}
            },
        );
        const root = document.createElement("div");
        const inView = document.createElement("div");
        const outOfView = document.createElement("div");
        const onenter = vi.fn();

        ViewObserver.create(root, onenter, 2000);
        report(
            [
                { isIntersecting: true, target: inView } as unknown as IntersectionObserverEntry,
                { isIntersecting: false, target: outOfView } as unknown as IntersectionObserverEntry,
            ],
            {} as IntersectionObserver,
        );

        expect(seen).toEqual([{ root, rootMargin: "2000px 0px" }]);
        expect(onenter).toHaveBeenCalledExactlyOnceWith(inView);
    });
});
