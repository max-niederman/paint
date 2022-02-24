import { writable, Writable } from "svelte/store";
import { authToken, getAuth } from "./auth";
import deepmerge from "deepmerge";

export async function makeCanvasRequest<T>(view: Oil.View, path: string, init?: RequestInit): Promise<T> {
	// TODO: should Oil consult its database to get the base URL and access token?
	//       this would remove the need to share the token with the client as well as
	//       allowing greater flexibility to cache Canvas resources on our backend,
	//       but also increase database load and worsen performance.

	const response = await fetch(
		`https://cors-anywhere.herokuapp.com/${view.canvas_base_url}${path}`,
		deepmerge(
			{
				headers: {
					Authorization: `Bearer ${view.canvas_access_token}`
				}
			},
			init ?? {}
		)
	);

	if (!response.ok) {
		throw new Error("response was not ok");
	}

	return await response.json();
}

export async function makeCanvasRequestPaginated<T>(view: Oil.View, path: string, init?: RequestInit): Promise<T[]> {
	let next = `${view.canvas_base_url}${path}`;
	let items: T[] = [];
	pages: while (true) {
		const response = await fetch(
			`https://cors-anywhere.herokuapp.com/${next}`,
			deepmerge(
				{
					headers: {
						Authorization: `Bearer ${view.canvas_access_token}`
					}
				},
				init ?? {}
			)
		);

		if (!response.ok) {
			throw new Error("response was not ok");
		}

		items = items.concat(await response.json());

		const linkHeader = response.headers.get("Link");
		const links = linkHeader.split(",");

		for (const link of links) {
			let [url, rel] = link.split(";").map((s) => s.trim());

			url = url.slice(1, -1); // remove brackets
			rel = rel.slice(5, -1); // remove `rel="X"`

			if (rel === "next") {
				next = url;
				continue pages;
			}
		}

		return items;
	}
}

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
