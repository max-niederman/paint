import { writable, Writable } from "svelte/store";
import { authToken } from "./auth";

interface View extends Oil.View {

}

export const view: Writable<View> = writable(null);
export const views: Writable<View[]> = writable([]);

authToken.subscribe(async (token) => {
    if (token) {
        const resp = await fetch(`${import.meta.env.VITE_OIL_URL}/views`, {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });
        const body = await resp.json();
        views.set(body);
        view.set(body[0]);
    }
});