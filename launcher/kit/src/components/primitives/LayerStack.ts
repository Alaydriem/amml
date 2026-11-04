/** Handler of the topmost open layer. */
export type LayerKeyHandler = (event: KeyboardEvent) => void;

interface Layer {
    token: symbol;
    onkeydown: LayerKeyHandler;
}

/**
 * The open layers of the page (dialogs, menus) in the order they opened. One `keydown` listener on the window
 * passes each key to the top layer only, so Escape and Tab never reach a layer below it.
 */
export class LayerStack {
    private static readonly layers: Layer[] = [];

    private static readonly dispatch = (event: KeyboardEvent): void => {
        LayerStack.layers[LayerStack.layers.length - 1]?.onkeydown(event);
    };

    /** Opens a layer on top of the stack. Pass the returned token to `close`. */
    static open(onkeydown: LayerKeyHandler): symbol {
        const token = Symbol("layer");
        if (LayerStack.layers.length === 0) window.addEventListener("keydown", LayerStack.dispatch);
        LayerStack.layers.push({ token, onkeydown });
        return token;
    }

    /** Removes a layer wherever it sits in the stack. A token that is not open is ignored. */
    static close(token: symbol): void {
        const index = LayerStack.layers.findIndex((layer) => layer.token === token);
        if (index < 0) return;
        LayerStack.layers.splice(index, 1);
        if (LayerStack.layers.length === 0) window.removeEventListener("keydown", LayerStack.dispatch);
    }

    static isTop(token: symbol): boolean {
        return LayerStack.layers[LayerStack.layers.length - 1]?.token === token;
    }

    static get depth(): number {
        return LayerStack.layers.length;
    }
}
