import { writable, Writable } from "svelte/store";
import { authToken, getAuth } from "./auth";
import deepmerge from "deepmerge";

class LocalStorageKey<T> {
	constructor(private key: string) {
		this.key = key;
	}

	get(defaultVal?: T): T {
		const val = localStorage.getItem(this.key);
		if (val == null) {
			return defaultVal;
		}

		try {
			return JSON.parse(val);
		} catch {
			return defaultVal;
		}
	}

	set(val: T): void {
		localStorage.setItem(this.key, JSON.stringify(val));
	}

	subscribeTo(store: SvelteStore<T>): void {
		store.subscribe((val) => {
			// for whatever reason the store was sometimes calling subscribers with undefined on subscription
			// this avoids writing undefined to local storage
			if (val !== undefined) {
				this.set(val);
			}
		});
	}
}

// persist the view in localStorage
const viewLSKey = new LocalStorageKey<Oil.View>("view");
export const view = writable(viewLSKey.get(null));
viewLSKey.subscribeTo(view);

// persist the views in localStorage
const viewsLSKey = new LocalStorageKey<Oil.View[]>("views");
export const views = writable(viewsLSKey.get([]));
viewsLSKey.subscribeTo(views);

// fetch token on login
authToken.subscribe(async (token) => {
	if (token) {
		const resp = await fetch(`${import.meta.env.VITE_OIL_URL}/views`, {
			headers: {
				Authorization: `Bearer ${token}`
			}
		});
		const body: Oil.View[] = await resp.json();
		views.set(body);
	}
});

// if we don't have a view set, set it to the first view available
views.subscribe((views) => {
	if (views.length > 0) {
		view.update((view) => (view === null ? views[0] : view));
	}
});
