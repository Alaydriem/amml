import type { RegistryView } from "#lib/bindings/RegistryView.ts";

// A stand-in for the registry event stream: the test emits lists by hand.
export class FakeRegistryEvents {
    private readonly receivers: ((views: RegistryView[]) => void)[] = [];
    unsubscribed = 0;

    subscribe = async (receive: (views: RegistryView[]) => void): Promise<() => void> => {
        this.receivers.push(receive);
        return () => {
            this.unsubscribed += 1;
            const at = this.receivers.indexOf(receive);
            if (at >= 0) this.receivers.splice(at, 1);
        };
    };

    emit(views: RegistryView[]): void {
        for (const receive of [...this.receivers]) receive(views);
    }

    get listeners(): number {
        return this.receivers.length;
    }
}
