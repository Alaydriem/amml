import type { SentinelCallback } from "../../../src/components/catalog/SentinelCallback";
import type { SentinelObserverFactory } from "../../../src/components/catalog/SentinelObserverFactory";

/** An observer the test drives: it records what the list observes and reports an element on demand. */
export class FakeObserver {
    observed = new Set<Element>();
    onenter: SentinelCallback = () => {};
    marginPx: number | undefined;

    readonly factory: SentinelObserverFactory = (_root, onenter, marginPx) => {
        this.onenter = onenter;
        this.marginPx = marginPx;
        return {
            observe: (element) => this.observed.add(element),
            unobserve: (element) => this.observed.delete(element),
            disconnect: () => this.observed.clear(),
        };
    };

    /** The observed element that stands for `page`; the first one when several groups share the page. */
    sentinel(page: number, index = 0): Element {
        const found = [...this.observed].filter((element) => element.getAttribute("data-page") === String(page));
        const element = found[index];
        if (!element) throw new Error(`no sentinel ${index} for page ${page}`);
        return element;
    }

    sentinels(page: number): Element[] {
        return [...this.observed].filter((element) => element.getAttribute("data-page") === String(page));
    }

    enter(page: number, index = 0) {
        this.onenter(this.sentinel(page, index));
    }
}
