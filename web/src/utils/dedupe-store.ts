import { Readable, writable } from "svelte/store";

export default function dedupe<T>(store: Readable<T>, eq?: (a: T, b: T) => boolean): Readable<T> {
	eq ??= (a, b) => a === b;

	let lastValue: T;
	const dedupedStore = writable(lastValue);
	store.subscribe((val) => {
		if (!eq(val, lastValue)) {
			lastValue = val;
			dedupedStore.set(val);
		}
	});

	return {
		subscribe: dedupedStore.subscribe
	};
}
