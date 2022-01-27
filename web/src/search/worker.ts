import initWasm, { SearchManager } from "glaze-wasm";
import type { View, Query, QueryResults } from "glaze-wasm";

export type Request =
	| {
			type: "update";
			view: View;
	  }
	| {
			type: "query";
			view: View;
			query: Query;
	  };

export type Response =
	| {
			type: "update";
	  }
	| {
			type: "query";
			results: QueryResults;
	  };

let initialized = false;
let searchManager: SearchManager;

async function ensureInitialized() {
	if (!initialized) {
		console.log("initializing search worker");
		await initWasm();
		searchManager = new SearchManager();
		initialized = true;
	}
}

self.onmessageerror = console.error;
self.onmessage = async (e) => {
	let req: Request = e.data;
	console.log(`recieved request: ${req}`);

	await ensureInitialized();

	switch (req.type) {
		case "update":
			searchManager.update(req.view);
			postMessage({ type: "update" });
			break;

		case "query":
			postMessage({ type: "query", results: searchManager.query(req.view, req.query) });
			break;

		default:
			const invalidReq: never = req;
			throw new Error(`Invalid request: ${invalidReq}`);
	}
};
