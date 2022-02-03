import initWasm, { SearchManager } from "glaze-wasm";
import WasmURL from "glaze-wasm/glaze_wasm_bg.wasm";
import type { View, Query, QueryResults } from "glaze-wasm";

export type Request =
	| {
			type: "update";
			view: View;
			since: string;
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

export interface SearchWorker extends Worker {
	postMessage(req: Request): void;
	onmessage: (e: MessageEvent<Response>) => void;
}

let initialized = false;
let searchManager: SearchManager;

async function ensureInitialized() {
	if (!initialized) {
		console.log("initializing search worker");
		await initWasm(WasmURL);
		searchManager = await new SearchManager();
		initialized = true;
	}
}

self.onmessageerror = console.error;
self.onmessage = async (e) => {
	let req: Request = e.data;
	console.log(`recieved request of type '${req.type}' from main thread`);

	await ensureInitialized();

	switch (req.type) {
		case "update":
			await searchManager.update(req.view, req.since);
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
