import type { SentinelObserverFactory } from "./SentinelObserverFactory";

/** The real observer: reports an element when it starts to intersect the scroll container and its margin. */
export class ViewObserver {
    /** The options of the underlying `IntersectionObserver`. */
    static options(root: Element | null, marginPx: number): IntersectionObserverInit {
        return { root, rootMargin: `${Math.max(0, Math.round(marginPx))}px 0px` };
    }

    static readonly create: SentinelObserverFactory = (root, onenter, marginPx) => {
        const observer = new IntersectionObserver((entries) => {
            for (const entry of entries) if (entry.isIntersecting) onenter(entry.target);
        }, ViewObserver.options(root, marginPx));
        return {
            observe: (element) => observer.observe(element),
            unobserve: (element) => observer.unobserve(element),
            disconnect: () => observer.disconnect(),
        };
    };
}
