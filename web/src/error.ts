import { writable, Writable } from "svelte/store";

export type Error =
	| {
			type: "not_found";
			help?: string;
	  }
	| {
			type: "server_error";
			body: any;
	  }
	| {
			type: "unknown_http_status";
			status: number;
            resp: Response;
	  };

export const error: Writable<Error> = writable(null);
