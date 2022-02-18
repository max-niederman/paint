import { writable, Writable } from "svelte/store";
import { authToken } from "./auth";
import deepmerge from "deepmerge";

export async function makeCanvasRequest<T>(view: Oil.View, path: string, init?: RequestInit): Promise<T> {
	const response = await fetch(
		`${view.canvas_base_url}${path}`,
		deepmerge(
			{
				headers: {
					Authorization: `Bearer ${view.canvas_access_token}`
				}
			},
			init ?? {}
		)
	);
	console.log(response);
	return await response.json();
}

// persist the view in localStorage
export const view: Writable<Oil.View> = writable(JSON.parse(localStorage.getItem("view") ?? "null"));
view.subscribe((view) => localStorage.setItem("view", JSON.stringify(view)));

// persist the views in localStorage
export const views: Writable<Oil.View[]> = writable(JSON.parse(localStorage.getItem("views") ?? "[]"));
views.subscribe((views) => localStorage.setItem("views", JSON.stringify(views)));

// fetch views from the server
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
