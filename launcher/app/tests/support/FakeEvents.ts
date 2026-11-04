import type { JobSnapshot } from "#lib/bindings/JobSnapshot.ts";

// A stand-in for the job event stream: the test emits snapshots by hand.
export class FakeEvents {
    private readonly receivers: ((snapshot: JobSnapshot) => void)[] = [];
    unsubscribed = 0;

    subscribe = async (receive: (snapshot: JobSnapshot) => void): Promise<() => void> => {
        this.receivers.push(receive);
        return () => {
            this.unsubscribed += 1;
            const at = this.receivers.indexOf(receive);
            if (at >= 0) this.receivers.splice(at, 1);
        };
    };

    emit(snapshot: JobSnapshot): void {
        for (const receive of [...this.receivers]) receive(snapshot);
    }

    get listeners(): number {
        return this.receivers.length;
    }

    // lets promises that wait on the subscription run
    static async settle(): Promise<void> {
        for (let turn = 0; turn < 5; turn += 1) await Promise.resolve();
    }
}
