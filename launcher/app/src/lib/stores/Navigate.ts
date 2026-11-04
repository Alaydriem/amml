// Goes to a route. A route passes SvelteKit's `goto`; a test passes a recorder.
export type Navigate = (path: string) => void;
